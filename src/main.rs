extern crate regex;

extern crate toml;
#[macro_use]
extern crate serde_derive;

extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_s3;

use std::process;
use std::env;

mod endpoint;

fn main() {
    let config_file = match env::args().nth(1) {
        Some(c) => c,
        None => {
            println!("Usage: {} <config_toml>", env::args().nth(1).unwrap_or("hpkgserve".to_string()));
            process::exit(1);
        }
    };
    let endpoints = match endpoint::from_file(config_file) {
		Ok(o) => o,
		Err(e) => {
			println!("Error: {}", e);
			process::exit(1);
		}
	};

	for (name, config) in endpoints.iter() {
		println!("key: {} val: {:?}", name, config);
		match endpoint::process_s3(&config){
			Ok(_) => {},
			Err(e) => {
				println!("Error: {}", e);
				process::exit(1);
			}
		}
	}
}
