/*
 * Copyright, 2018-2022 Haiku, Inc., All rights reserved.
 * Released under the terms of the MIT license
 *
 * Authors:
 *      Alexander von Gluck IV
 *      Niels Sascha Reedijk
 */

extern crate natord;

extern crate regex;

#[macro_use]
extern crate rocket;
extern crate rocket_prometheus;

extern crate toml;
extern crate s3;

extern crate url;

use std::sync::{Arc,Mutex};
use std::{process,thread};
//use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use rocket::State;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::request::Request;
use rocket_prometheus::PrometheusMetrics;

mod routecache;

#[catch(404)]
fn sys_not_found(_req: &Request) -> String {
    format!("Sorry, that's not a valid path!")
}

#[get("/healthz")]
fn sys_health(_cachedb: &State<Arc<Mutex<routecache::RouteCache>>>) -> (Status, String) {
    (Status::Ok, "{{\"status\": \"OK\"}}".to_string())

    // TODO: Report last cache rebuild time?
    // TODO: Check for issues, report unhealthy?
    //response.set_sized_body(None, Cursor::new(format!("Fatal: Cache Sync Failure: {}", e)));
    //response.set_status(Status::InternalServerError);
}

#[get("/")]
fn index(cachedb: &State<Arc<Mutex<routecache::RouteCache>>>) -> (Status, String) {
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let branches = cache.branches();
    (Status::Ok, format!("{:?}", branches))
}

#[get("/<branch>")]
fn index_branch(cachedb: &State<Arc<Mutex<routecache::RouteCache>>>, branch: &str) -> (Status, String) {
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let arches = cache.architectures(branch.to_string());
    (Status::Ok, format!("{:?}", arches))
}

#[get("/<branch>/<arch>")]
fn index_arch(cachedb: &State<Arc<Mutex<routecache::RouteCache>>>, branch: &str, arch: &str) -> (Status, String) {
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let versions = cache.versions(branch.to_string(), arch.to_string());
    (Status::Ok, format!("{:?}", versions))
}

#[get("/<branch>/<arch>/<version>", rank = 1)]
fn index_repo<'a>(cachedb: &'a State<Arc<Mutex<routecache::RouteCache>>>, branch: &str, arch: &str, version: &str) -> (Status, String) {
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let repo = match cache.lookup_repo(branch.to_string(), arch.to_string(), version.to_string()) {
        Some(r) => r,
        None => {
            return (Status::NotFound, "Invalid repository".to_string());
        }
    };
    (Status::Ok, format!("{:?}", repo))
}

#[get("/<branch>/<arch>/<version>/<path..>", rank = 2)]
fn access_repo<'a>(cachedb: &'a State<Arc<Mutex<routecache::RouteCache>>>, branch: &str, arch: &str, version: &str, path: PathBuf) -> Redirect {
    // TODO: Handle lock error, return failure?
    let mut cache = cachedb.lock().unwrap();

    let prefix_url = cache.public_prefix().unwrap();
    let repo_file = path.to_str().unwrap();
    let repo = match cache.lookup_repo(branch.to_string(), arch.to_string(), version.to_string()) {
        Some(r) => r,
        None => return Redirect::to(format!("..")),
    };

    let final_url = prefix_url.join(format!("{}/{}", repo.path, repo_file).as_str()).unwrap();
    Redirect::to(final_url.to_string())
}


#[launch]
fn rocket() -> _ {
    let mut config = match routecache::RouteConfig::init() {
        Ok(c) => {
            println!("Server has been configured.");
            Some(c)
        },
        Err(e) => {
            println!("Error in configuration: {}", e);
            None
        },
    };

    // Make sure we have a valid cache before handling requests.
    let mut cache = routecache::RouteCache::new(config.unwrap());
    match cache.sync() {
        Ok(_) => {},
        Err(e) => println!("Cache Sync Error: {}", e),
    };

    let prometheus = PrometheusMetrics::new();

    // This mutex gets consumed by multiple threads (the cache rebuilder, and every route.
    let cache_state = Arc::new(Mutex::new(cache));

    // Trigger a check of our repo cache every 60 seconds to see if it needs rebuilt.
    let tcache = cache_state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            let mut c = tcache.lock().unwrap();
            match c.sync() {
                Ok(_) => {},
                Err(e) => println!("Sync: Error: {}", e),
            }
        }
    });

    // Launch our web server, begin serving requests
    rocket::build()
        .manage(cache_state.clone())
        .attach(prometheus.clone())
        .mount("/", routes![sys_health, index, index_branch, index_arch, index_repo, access_repo])
        .mount("/metrics", prometheus)
        .register("/", catchers![sys_not_found])
}
