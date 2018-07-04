
use toml;
use std::fs::File;
use std::path::{Path,PathBuf};
use std::error::Error;
use std::io::Read;
use std::collections::HashMap;

use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::region::Region;

#[derive(Debug, Deserialize)]
pub struct Endpoint {
    pub public_url: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub s3_secret: Option<String>,
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Endpoint>, Box<Error>> {
    let mut fh = File::open(path.as_ref())?;
    let mut buffer = String::new();
    fh.read_to_string(&mut buffer)?;
    let decoded: HashMap<String, Endpoint> = toml::from_str(buffer.as_str())?;
    return Ok(decoded);
}

pub fn process(endpoint: &Endpoint) -> Result<String, Box<Error>> {
	let s3_endpoint = match &endpoint.s3_endpoint {
		Some(s) => s,
		None => {
			return Err(From::from("s3_endpoint missing"));
		}
	};
	let s3_bucket = match &endpoint.s3_bucket {
		Some(s) => s,
		None => {
			return Err(From::from("s3_bucket missing"));
		}
	};
	let s3_key = endpoint.s3_key.clone();
	let s3_secret = endpoint.s3_secret.clone();

	let credentials = Credentials::new(s3_key, s3_secret, None, None);
	let region: Region = s3_endpoint.parse()?;
	let bucket = Bucket::new(&s3_bucket, region, credentials);

	println!("{:?}", bucket);

	return Ok("ok".to_string())
}
