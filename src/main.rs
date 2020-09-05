/*
 * Copyright, 2018-2020 Haiku, Inc., All rights reserved.
 * Released under the terms of the MIT license
 *
 * Authors:
 *      Alexander von Gluck IV
 *      Niels Sascha Reedijk
 */

#![feature(proc_macro_hygiene, decl_macro)]

extern crate natord;

extern crate regex;

#[macro_use]
extern crate rocket;

extern crate toml;
extern crate s3;

extern crate url;

use std::sync::{Arc,Mutex};
use std::{process,thread};
//use std::error::Error;
use std::path::PathBuf;
use std::io::Cursor;
use std::time::Duration;

use rocket::State;
use rocket::http::Status;
use rocket::response::{Response, Redirect};
use rocket::request::Request;

mod routecache;

#[catch(404)]
fn sys_not_found(_req: &Request) -> String {
    format!("Sorry, that's not a valid path!")
}

#[get("/healthz")]
fn sys_health(_cachedb: State<Arc<Mutex<routecache::RouteCache>>>) -> Response {
    let mut response = Response::new();

    // TODO: Report last cache rebuild time?
    // TODO: Check for issues, report unhealthy?
    //response.set_sized_body(Cursor::new(format!("Fatal: Cache Sync Failure: {}", e)));
    //response.set_status(Status::InternalServerError);

    response.set_sized_body(Cursor::new(format!("{{\"status\": \"OK\"}}")));
    response
}

#[get("/")]
fn index(cachedb: State<Arc<Mutex<routecache::RouteCache>>>) -> Response {
    let mut response = Response::new();
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let branches = cache.branches();
    response.set_sized_body(Cursor::new(format!("{:?}", branches)));
    response
}

#[get("/<branch>")]
fn index_branch(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String) -> Response {
    let mut response = Response::new();
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let arches = cache.architectures(branch);
    response.set_sized_body(Cursor::new(format!("{:?}", arches)));
    response
}

#[get("/<branch>/<arch>")]
fn index_arch(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String) -> Response {
    let mut response = Response::new();
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let versions = cache.versions(branch, arch);
    response.set_sized_body(Cursor::new(format!("{:?}", versions)));
    response
}

#[get("/<branch>/<arch>/<version>")]
fn index_repo(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String, version: String) -> Response {
    let mut response = Response::new();
    // TODO: Handle lock error, return failure
    let mut cache = cachedb.lock().unwrap();
    let repo = match cache.lookup_repo(branch.clone(), arch.clone(), version.clone()) {
        Some(r) => r,
        None => {
            response.set_sized_body(Cursor::new(format!("Invalid repository!")));
            response.set_status(Status::NotFound);
            return response;
        }
    };
    response.set_sized_body(Cursor::new(format!("{:?}", repo)));
    response
}

#[get("/<branch>/<arch>/<version>/<path..>")]
fn access_repo(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String, version: String, path: PathBuf) -> Redirect {
    // TODO: Handle lock error, return failure?
    let mut cache = cachedb.lock().unwrap();

    let prefix_url = cache.public_prefix().unwrap();
    let repo_file = path.to_str().unwrap();
    let repo = match cache.lookup_repo(branch.clone(), arch.clone(), version.clone()) {
        Some(r) => r,
        None => return Redirect::to(format!("..")),
    };

    let final_url = prefix_url.join(format!("{}/{}", repo.path, repo_file).as_str()).unwrap();
    Redirect::to(final_url.to_string())
}


fn main() {

    // Check for Docker / Kubernetes secrets first.
    let mut config = match routecache::RouteConfig::new_from_secrets() {
        Ok(c) => {
            println!("Found configuration secrets at /run/secrets.");
            Some(c)
        },
        Err(e) => {
            println!("Didn't find valid secrets: {}", e);
            None
        },
    };

    // If we can't locate Docker secrets, look at environment vars.
    if config.is_none() {
        config = match routecache::RouteConfig::new_from_env() {
            Ok(c) => {
                println!("Found environment-based configuration!");
                Some(c)
            },
            Err(e) => {
                println!("Error: {}", e);
                process::exit(1);
            },
        };
    };

    // Make sure we have a valid cache before handling requests.
    let mut cache = routecache::RouteCache::new(config.unwrap());
    match cache.sync() {
        Ok(_) => {},
        Err(e) => println!("Cache Sync Error: {}", e),
    };


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
    rocket::ignite()
        .manage(cache_state.clone())
        .mount("/", routes![sys_health, index, index_branch, index_arch, index_repo, access_repo])
        .register(catchers![sys_not_found])
        .launch();
}
