use std::env;

use std::error::Error;
use std::time::{Duration, Instant};
use std::fs::File;
use std::path::Path;

use s3::bucket::Bucket;
use s3::region::Region;
use s3::credentials::Credentials;
use s3::error::S3Error;

#[derive(Clone, Debug)]
pub struct RouteConfig {
    pub cache_ttl: u64,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub s3_secret: Option<String>,
    pub s3_prefix: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Route {
    pub branch: String,
    pub arch: String,
    pub version: String,
}

#[derive(Clone, Debug)]
pub struct RouteCache {
    pub last_update: Option<Instant>,
    pub routes: Vec<Route>,
    pub config: RouteConfig,
}

impl PartialEq for Route {
    fn eq(&self, other: &Route) -> bool {
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

impl RouteConfig {
    pub fn new() -> RouteConfig {
        RouteConfig {
            cache_ttl: 30,
            s3_region: None,
            s3_endpoint: None,
            s3_bucket: None,
            s3_key: None,
            s3_secret: None,
            s3_prefix: None,
        }
    }

    pub fn new_from_env() -> Result<RouteConfig, Box<dyn Error>> {
        let mut config = RouteConfig::new();

        // Optional
        config.cache_ttl = match env::var("CACHE_TTL") {
            Ok(v) => v.parse::<u64>()?,
            Err(_) => 30,
        };
        config.s3_region = match env::var("S3_REGION") {
            Ok(v) => Some(v),
            Err(_) => Some("us-east-1".to_string()),
        };
        config.s3_prefix = match env::var("S3_PREFIX") {
            Ok(v) => Some(v),
            Err(_) => Some("".to_string()),
        };

        // Required
        config.s3_endpoint = match env::var("S3_ENDPOINT") {
            Ok(v) => Some(v),
            Err(_) => return Err(From::from("S3_ENDPOINT environment variable is unset!")),
        };
        config.s3_bucket = match env::var("S3_BUCKET") {
            Ok(v) => Some(v),
            Err(_) => return Err(From::from("S3_BUCKET environment variable is unset!")),
        };
        config.s3_key = match env::var("S3_KEY") {
            Ok(v) => Some(v),
            Err(_) => return Err(From::from("S3_KEY environment variable is unset!")),
        };
        config.s3_secret = match env::var("S3_SECRET") {
            Ok(v) => Some(v),
            Err(_) => return Err(From::from("S3_SECRET environment variable is unset!")),
        };
        return Ok(config);
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

    pub fn sync(&mut self) -> Result<usize, Box<dyn Error>> {
        let config = self.config.clone();

        if self.last_update != None && self.last_update.unwrap().elapsed().as_secs() < config.cache_ttl {
            return Ok(0);
        }

        println!("RouteCache/Sync: TTL expired, refreshing bucket inventory...");

        let region = Region::Custom {
            region: config.s3_region.unwrap(),
            endpoint: config.s3_endpoint.unwrap(),
        };
        let credentials = Credentials::new(config.s3_key, config.s3_secret, None, None);
        let bucket = Bucket::new(&config.s3_bucket.unwrap(), region, credentials)?;

        //let mut architectures: Vec<Architecture> = Vec::new();
        let base_prefix = config.s3_prefix.unwrap().clone();
        let results = bucket.list(&base_prefix, None)?;
        let mut routes: Vec<Route> = Vec::new();
        for (list, code) in results {
            for object in list.contents {
                let mut fields = object.key.split("/");
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
                let route = Route{branch: branch, arch: arch, version: version};
                if !routes.contains(&route) {
                    routes.push(route);
                }
            }
        }
        println!("RouteCache/Sync: Complete. {} resources located.", routes.len());
        self.routes = routes;
        self.last_update = Some(Instant::now());
        return Ok(0);
    }

    pub fn latest_version(&mut self, branch: String, arch: String) {
        self.sync();
        let mut potential_routes: Vec<Route> = Vec::new();
        for route in self.routes.iter() {
            if route.branch == branch && route.arch == arch {
                potential_routes.push(route.clone());
            }
        }
        println!("I found these versions for {} {}:", branch, arch);
        for i in potential_routes {
            println!("{:?}", i);
        }
    }
}
