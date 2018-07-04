extern crate s3;
extern crate toml;
#[macro_use]
extern crate serde_derive;

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
		match endpoint::process(&config){
			Ok(_) => {},
			Err(e) => {
				println!("Error: {}", e);
				process::exit(1);
			}
		}
	}
}
