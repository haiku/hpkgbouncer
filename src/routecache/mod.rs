/*
 * Copyright, 2018-2022 Haiku, Inc., All rights reserved.
 * Released under the terms of the MIT license
 *
 * Authors:
 *      Alexander von Gluck IV
 *      Niels Sascha Reedijk
 */

use std::{env,fs};

use std::error::Error;
use std::time::Instant;
use std::cmp::Ordering;

use natord::compare;

use s3::bucket::Bucket;
use s3::region::Region;
use s3::creds::Credentials;

use url::Url;

#[derive(Clone, Debug)]
pub struct RouteConfig {
    pub s3_public: Option<String>,
    pub cache_ttl: u64,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub s3_secret: Option<String>,
    pub s3_prefix: Option<String>,
}

#[derive(Clone, Debug, Eq)]
pub struct Route {
    pub branch: String,
    pub arch: String,
    pub version: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct RouteCache {
    pub last_update: Option<Instant>,
    pub routes: Vec<Route>,
    pub config: RouteConfig,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        if self.branch != other.branch {
            return false;
        }
        if self.arch != other.arch {
            return false;
        }
        if self.version != other.version {
            return false;
        }
        return true;
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(compare(&self.version, &other.version))
    }
}

impl Ord for Route {
    fn cmp(&self, other: &Route) -> Ordering {
        compare(&self.version, &other.version)
    }
}

fn sanitize_string(input: String) -> String {
    input.replace("\n", "")
}

fn read_from_secrets(name: &str, required: bool) -> Result<Option<String>, Box<dyn Error>> {
    let metadata = fs::metadata(format!("/run/secrets/{}", name));
    if required && metadata.is_err() {
      return Err(From::from(format!("{} secret is unset.", name)));
    }
    let raw = match fs::read_to_string(format!("/run/secrets/{}", name)) {
        Ok(o) => o,
        Err(_) => {
            if required {
                return Err(From::from(format!("Unable to read secret {}", name)));
            }
            return Ok(None);
        }
    };
    match raw.parse() {
        Ok(o) => Ok(Some(sanitize_string(o))),
        Err(_) => {
            if required {
                return Err(From::from(format!("Unable to parse secret {}", name)));
            }
            Ok(None)
        }
    }
}

fn read_from_env(name: &str, required: bool) -> Result<Option<String>, Box<dyn Error>> {
    match env::var(name) {
        Ok(v) => Ok(Some(sanitize_string(v))),
        Err(_) => {
            if required {
                return Err(From::from(format!("{} environment variable is unset!", name)));
            }
            Ok(None)
        }
    }
}


impl RouteConfig {
    pub fn new() -> RouteConfig {
        RouteConfig {
            s3_public: None,
            cache_ttl: 900,
            s3_region: Some("".to_string()),
            s3_endpoint: None,
            s3_bucket: None,
            s3_key: None,
            s3_secret: None,
            s3_prefix: None,
        }
    }

    pub fn init() -> Result<RouteConfig, Box<dyn Error>> {
        let mut config = RouteConfig::new();

        // Check for required secrets
        config.s3_endpoint = read_from_secrets("s3_endpoint", true)?;
        config.s3_bucket = read_from_secrets("s3_bucket", true)?;
        config.s3_key = read_from_secrets("s3_key", true)?;
        config.s3_secret = read_from_secrets("s3_secret", true)?;
        config.s3_region = read_from_secrets("s3_region", false)?;
        config.overlay_env()?;

        println!("{:?}", config);

        return Ok(config);
    }

    fn overlay_env(&mut self) -> Result<(), Box<dyn Error>> {
        // optional env vars
        self.s3_prefix = read_from_env("S3_PREFIX", false)?;
        self.s3_public = read_from_env("S3_PUBLIC", false)?;

        let cache_ttl = read_from_env("CACHE_TTL", false)?;
        self.cache_ttl = cache_ttl.unwrap_or("900".to_string()).parse::<u64>()?;

        return Ok(());
    }

}

impl RouteCache {
    pub fn new(config: RouteConfig) -> RouteCache {
        let routes: Vec<Route> = Vec::new();
        let route_cache = RouteCache {
            last_update: None,
            routes: routes,
            config: config,
        };
        return route_cache;
    }

    fn is_route_ready(&mut self, route: &Route, bucket: &Bucket) -> Result<bool, Box<dyn Error>> {
        let requirements = vec!["repo".to_string(), "repo.sha256".to_string()];

        let results = bucket.list_blocking(route.path.clone(), None)?;
        let mut keys: Vec<String> = Vec::new();

        // collect the contents of the folder
        for list in results {
            for object in list.contents {
                let name = object.key.strip_prefix(&route.path).unwrap_or("");
                keys.push(name.to_string());
            }
        }
        // Now, validate each file exists
        for filename in requirements.iter() {
            if !keys.contains(filename) {
                println!("{} is not ready yet (missing {})", route.path, filename);
                return Ok(false);
            }
        }
        return Ok(true);
    }

    pub fn sync(&mut self) -> Result<usize, Box<dyn Error>> {
        let config = self.config.clone();

        if self.last_update != None && self.last_update.unwrap().elapsed().as_secs() < config.cache_ttl {
            return Ok(0);
        }

        println!("RouteCache/Sync: TTL expired, refreshing bucket inventory...");

        let region = Region::Custom {
            region: config.s3_region.unwrap_or("us-east-1".to_string()),
            endpoint: config.s3_endpoint.unwrap(),
        };

        let credentials = Credentials::new(config.s3_key.as_deref(), config.s3_secret.as_deref(),
            None, None, None)?;
        let bucket = Bucket::new(&config.s3_bucket.unwrap(), region, credentials)?;

        // take prefix, and normalize it without the trailing /
        let base_prefix = match config.s3_prefix {
            Some(x) => format!("{}/", x.trim_end_matches("/").to_string()),
            None => "".to_string(),
        };

        let results = bucket.list_blocking(base_prefix.clone(), None)?;

        let mut valid_routes: Vec<Route> = Vec::new();
        let mut examined_routes: Vec<Route> = Vec::new();

        for list in results {
            for object in list.contents {
                // trim any potential prefix paths for simple matching
                let key = object.key.trim_start_matches(&base_prefix).to_string();

                let mut fields = key.split("/");
                println!("{} vs {}", object.key, key);
                //advance cursor past prefix
                //fields.nth(prefix_fields);

                // We're only interested in the repo within branch/arch/version folders
                // This cuts down scan time as we don't care about packages, etc
                let field_count = fields.clone().count();
                if field_count != 4 && fields.nth(4) != Some("repo") {
                    continue
                }

                let branch = match fields.next() {
                    Some(b) => b.to_string(),
                    None => continue,
                };
                let arch = match fields.next() {
                    Some(a) => a.to_string(),
                    None => continue,
                };
                let version = match fields.next() {
                    Some(v) => v.to_string(),
                    None => continue,
                };
                let route = Route{
                    branch: branch.clone(),
                    arch: arch.clone(),
                    version: version.clone(),
                    path: format!("{}/{}/{}", branch, arch, version),
                };

                // We track in two stages since an examined route doesn't have to be a valid route
                if examined_routes.contains(&route) {
                    continue
                }
                examined_routes.push(route.clone());

                // XXX: We need to rethink this.. it's too damn slow and hammers S3.  We should
                // probably pass in a listing of all of the files and use the pre-formed list.. or
                // only validate the "current" route is ready

                //if self.is_route_ready(&route, &bucket)? {
                //    println!("Adding {:?}", route);
                    valid_routes.push(route)
                //}
            }
        }
        println!("RouteCache/Sync: Complete. {} resources located.", valid_routes.len());
        self.routes = valid_routes;
        self.last_update = Some(Instant::now());
        return Ok(0);
    }

    pub fn public_prefix(&mut self) -> Result<Url, Box<dyn Error>> {
        let mut base: String;

        if self.config.s3_public != None {
            base = self.config.s3_public.clone().unwrap();
        } else {
            base = format!("{}/{}/", self.config.s3_endpoint.clone().unwrap(),
                self.config.s3_bucket.clone().unwrap());
            match self.config.s3_prefix.clone() {
                None => {},
                Some(p) => {
                    if p.len() > 0 {
                        base.push_str(format!("{}/", p).as_str());
                    }
                }
            }
        }
        Ok(Url::parse(&base)?)
    }

    fn version_latest(&mut self, branch: String, arch: String) -> Option<Route> {
        let mut potential_routes: Vec<Route> = Vec::new();
        for route in self.routes.iter() {
            if route.branch == branch && route.arch == arch {
                potential_routes.push(route.clone());
            }
        }
        potential_routes.sort();
        return potential_routes.pop();
    }

    pub fn lookup_repo(&mut self, branch: String, arch: String, version: String) -> Option<Route> {
        // If asking for "current" version.
        if version == "current" {
            return self.version_latest(branch, arch);
        }

        // Otherwise just return requested version.
        for route in self.routes.iter() {
            if route.branch == branch && route.arch == arch && route.version == version {
                return Some(route.clone());
            }
        }
        return None;
    }

    pub fn branches(&mut self) -> Vec<String> {
        let mut branches: Vec<String> = Vec::new();
        for route in self.routes.iter() {
            if !branches.contains(&route.branch) {
                branches.push(route.branch.clone());
            }
        }
        branches
    }

    pub fn architectures(&mut self, branch: String) -> Vec<String> {
        let mut arches: Vec<String> = Vec::new();
        for route in self.routes.iter() {
            if route.branch != branch {
                continue;
            }
            if !arches.contains(&route.arch) {
                arches.push(route.arch.clone());
            }
        }
        arches
    }

    pub fn versions(&mut self, branch: String, arch: String) -> Vec<String> {
        let mut versions: Vec<String> = Vec::new();
        for route in self.routes.iter() {
            if route.branch != branch || route.arch != arch {
                continue;
            }
            if !versions.contains(&route.arch) {
                versions.push(route.version.clone());
            }
        }
        versions
    }
}
