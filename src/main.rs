extern crate regex;
extern crate futures;
extern crate hyper;

extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate s3;

extern crate url;

use std::{env, process};
use std::error::Error;

use futures::future;
use hyper::{Body, Method, Response, Request, Server, StatusCode};
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::header::{HeaderMap, LOCATION};

//use url::Url;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

mod routecache;

fn router(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());

    //let endpoint = match get_config() {
    //    Ok(o) => o,
    //    Err(e) => {
    //        println!("Error: {}", e);
    //        process::exit(1);
    //    }
    //};
    //let inventory = match endpoint::process_s3(&endpoint) {
    //    Ok(o) => o,
    //    Err(e) => {
	//		println!("Error: {}", e);
	//		process::exit(1);
	//	}
    //};
    //let architectures: Vec<String> = inventory.iter()
    //    .map(|i| i.prefix.clone().replace("/", ""))
    //    .collect();

    //match(req.method(), req.uri().path()) {
    //    (&Method::GET, "/") => {
    //        *response.body_mut() = Body::from(architectures.join("<br/>"));
    //    }

    //    (&Method::GET, _) => {
    //        let req_uri = req.uri().path().to_string();
    //        let req_parts: Vec<&str> = req_uri.split("/").filter(|v| v != &"").collect();
    //        if !architectures.contains(&req_parts.first().unwrap().to_string()) {
    //            *response.status_mut() = StatusCode::NOT_FOUND;
    //        } else {
    //            let base_pub_uri = format!("{}/{}", &endpoint.public_url, &endpoint.s3_bucket.unwrap());
    //            let mut final_url = String::new();
    //            if req_parts.last().unwrap() == &"current" {
    //                //final_url = format!("{}{}", base_pub_uri, inventory.latest.file);
    //                final_url = format!("{}/CATS", base_pub_uri);
    //            } else {
    //                final_url = format!("{}{}", base_pub_uri, req_uri);
    //            }
    //            let mut headers = HeaderMap::new();
    //            headers.insert(LOCATION, final_url.as_str().parse().unwrap());
    //            *response.headers_mut() = headers;
    //            *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;
    //        }
    //    }

    //    _ => {
    //        *response.status_mut() = StatusCode::NOT_FOUND;
    //    }
    //};
    Box::new(future::ok(response))
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
    cache.sync();

    // XXX: Testing
    let latest = cache.latest_version("master".to_string(), "x86_64".to_string()).unwrap();

    let port = env::var("LISTEN_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("LISTEN_PORT must be a number)");

    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::bind(&addr)
        .serve(|| service_fn(router))
        .map_err(|e| println!("server error: {}", e));

    println!("Server ready! Listening on 0.0.0.0:{} for requests!", port);
    hyper::rt::run(server);
}
