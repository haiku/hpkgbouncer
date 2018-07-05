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
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <config_toml> <region>", args[0]);
        process::exit(1);
    }
    let endpoints = match endpoint::from_file(args[1].clone()) {
		Ok(o) => o,
		Err(e) => {
			println!("Error: {}", e);
			process::exit(1);
		}
	};
    if !endpoints.contains_key(&args[2]) {
        println!("Error: config_toml doesn't contain region {}", args[2]);
        process::exit(1);
    }

    // Take inventory of what artifacts exist at endpoint
    let inventory = match endpoint::process_s3(&endpoints[&args[2]]) {
        Ok(o) => o,
        Err(e) => {
			println!("Error: {}", e);
			process::exit(1);
		}
    };
    println!("{:?}", inventory);
}
