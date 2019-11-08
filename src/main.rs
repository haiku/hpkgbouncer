#![feature(proc_macro_hygiene, decl_macro)]

extern crate regex;
extern crate futures;

#[macro_use]
extern crate rocket;

extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate s3;

extern crate url;

use std::sync::{Arc,Mutex};
use std::{env, process};
use std::error::Error;

use futures::future;
use rocket::State;
use rocket::response::Redirect;

//use url::Url;

mod routecache;

#[get("/")]
fn index(cachedb: State<Arc<Mutex<routecache::RouteCache>>>) -> String {
    let mut cache = cachedb.lock().unwrap();
    cache.sync();

    // let mut headers = HeaderMap::new();
    // headers.insert(LOCATION, final_url.as_str().parse().unwrap());
    // *response.headers_mut() = headers;
    // *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;

    // TODO: Dump all known branches
    let latest = cache.latest_version("master".to_string(), "x86_64".to_string()).unwrap();
    format!("{:?}", latest).to_string()
}

#[get("/<branch>")]
fn index_branch(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String) -> String {
    let mut cache = cachedb.lock().unwrap();
    cache.sync();

    // let mut headers = HeaderMap::new();
    // headers.insert(LOCATION, final_url.as_str().parse().unwrap());
    // *response.headers_mut() = headers;
    // *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;

    // TODO: Dump all known architectures
    let latest = cache.latest_version(branch, "x86_64".to_string()).unwrap();
    format!("{:?}", latest).to_string()
}

#[get("/<branch>/<arch>")]
fn index_arch(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String) -> String {
    let mut cache = cachedb.lock().unwrap();
    cache.sync();

    // let mut headers = HeaderMap::new();
    // headers.insert(LOCATION, final_url.as_str().parse().unwrap());
    // *response.headers_mut() = headers;
    // *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;

    // TODO: Dump all known architectures
    let latest = cache.latest_version(branch, arch).unwrap();
    format!("{:?}", latest).to_string()
}

#[get("/<branch>/<arch>/current")]
fn index_current(cachedb: State<Arc<Mutex<routecache::RouteCache>>>, branch: String, arch: String) -> Redirect {
    let mut cache = cachedb.lock().unwrap();
    cache.sync();

    // let mut headers = HeaderMap::new();
    // headers.insert(LOCATION, final_url.as_str().parse().unwrap());
    // *response.headers_mut() = headers;
    // *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;

    // TODO: Dump all known architectures
    let latest = cache.latest_version(branch, arch).unwrap();
    //format!("{:?}", latest).to_string()
    Redirect::to(format!("https://google.com/"))
}

fn main() {
    let config = match routecache::RouteConfig::new_from_env() {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        },
    };
    let mut cache = routecache::RouteCache::new(config);
    match cache.sync() {
        Ok(_) => {},
        Err(e) => println!("Cache Sync Error: {}", e),
    };

    rocket::ignite()
        .manage(Arc::new(Mutex::new(cache)))
        .mount("/", routes![index, index_branch, index_arch, index_current])
        .launch();

    //let router_service = move || {
    //    let mut service_cache = cache.clone();
    //    service_fn(move |mut req| {
    //        let mut response = Response::new(Body::empty());
    //        let branches = service_cache.branches();
    //        match(req.method(), req.uri().path()) {
    //            (&Method::GET, "/") => {
    //                *response.body_mut() = Body::from(format!("{:?}", branches));
    //            }

    //            (&Method::GET, _) => {
    //                let req_uri = req.uri().path().to_string();
    //                let req_parts: Vec<&str> = req_uri.split("/").filter(|v| v != &"").collect();
    //                let branch = &req_parts.first().unwrap().to_string();
    //                if !branches.contains(branch) {
    //                    *response.status_mut() = StatusCode::NOT_FOUND;
    //                } else {
    //                    let latest = service_cache.latest_version(branch.to_string(), "x86_64".to_string()).unwrap();
    //                    *response.body_mut() = Body::from(format!("{:?}", latest));
    //                }
    //            }

    //            _ => {
    //                *response.status_mut() = StatusCode::NOT_FOUND;
    //            }
    //        };
    //        Box::new(future::ok::<_, hyper::Error>(response))
    //    })
    //};
}
