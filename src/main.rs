#![feature(proc_macro_hygiene, decl_macro)]

extern crate natord;

extern crate regex;

#[macro_use]
extern crate rocket;

extern crate toml;
extern crate s3;

extern crate url;

use std::sync::{Arc,Mutex};
use std::{process};
//use std::error::Error;
use std::path::PathBuf;
use std::io::Cursor;

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
fn sys_health(_cachedb: State<Arc<Mutex<routecache::RouteCache>>>) -> String {
    // TODO: Maybe trigger cache rebuild?  Needs to be async vs sync though since a healthcheck
    // timeout could result in the service going unhealthy
    format!("{{\"status\": \"OK\"}}").to_string()
}

#[get("/")]
fn index(cachedb: State<Arc<Mutex<routecache::RouteCache>>>) -> Response {
    let mut cache = cachedb.lock().unwrap();
    let mut response = Response::new();
    match cache.sync() {
        Ok(_) => {},
        Err(e) => {
            response.set_sized_body(Cursor::new(format!("Fatal: Cache Sync Failure: {}", e)));
            response.set_status(Status::InternalServerError);
            return response;
        },
    };
    let branches = cache.branches();
    response.set_sized_body(Cursor::new(format!("{:?}", branches)));
    response
}

#[get("/<branch>")]
fn index_branch(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String) -> Response {
    let mut cache = cachedb.lock().unwrap();
    let mut response = Response::new();
    match cache.sync() {
        Ok(_) => {},
        Err(e) => {
            response.set_sized_body(Cursor::new(format!("Fatal: Cache Sync Failure: {}", e)));
            response.set_status(Status::InternalServerError);
            return response;
        },
    };
    let arches = cache.architectures(branch);
    response.set_sized_body(Cursor::new(format!("{:?}", arches)));
    response
}

#[get("/<branch>/<arch>")]
fn index_arch(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String) -> Response {
    let mut cache = cachedb.lock().unwrap();
    let mut response = Response::new();
    match cache.sync() {
        Ok(_) => {},
        Err(e) => {
            response.set_sized_body(Cursor::new(format!("Fatal: Cache Sync Failure: {}", e)));
            response.set_status(Status::InternalServerError);
            return response;
        },
    };
    let versions = cache.versions(branch, arch);
    response.set_sized_body(Cursor::new(format!("{:?}", versions)));
    response
}

#[get("/<branch>/<arch>/<version>")]
fn index_repo(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String, version: String) -> Response {
    let mut cache = cachedb.lock().unwrap();
    let mut response = Response::new();
    match cache.sync() {
        Ok(_) => {},
        Err(e) => {
            response.set_sized_body(Cursor::new(format!("Fatal: Cache Sync Failure: {}", e)));
            response.set_status(Status::InternalServerError);
            return response;
        },
    };
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
    let mut cache = cachedb.lock().unwrap();

    // TODO: Handle errors better!
    match cache.sync() {
        Ok(_) => {},
        Err(e) => {
            println!("Cache Sync Error: {}", e);
            return Redirect::to(format!(".."));
        },
    };

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

    let mut cache = routecache::RouteCache::new(config.unwrap());
    match cache.sync() {
        Ok(_) => {},
        Err(e) => println!("Cache Sync Error: {}", e),
    };

    rocket::ignite()
        .manage(Arc::new(Mutex::new(cache)))
        .mount("/", routes![sys_health, index, index_branch, index_arch, index_repo, access_repo])
        .register(catchers![sys_not_found])
        .launch();
}
