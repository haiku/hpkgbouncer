extern crate s3;
extern crate toml;
#[macro_use]
extern crate serde_derive;

use s3::bucket::Bucket;
use s3::credentials::Credentials;

mod endpointconfig;

fn main() {
    let endpoints = endpointconfig::from_file("config-sample.toml");
    println!("{:?}", endpoints.unwrap());
}
