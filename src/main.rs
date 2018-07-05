extern crate regex;

extern crate toml;
#[macro_use]
extern crate serde_derive;

extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_s3;

use std::process;

mod endpoint;

fn main() {
    let endpoints = match endpoint::from_file("config-sample.toml") {
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
