
use toml;
use std::fs::File;
use std::path::{Path,PathBuf};
use std::error::Error;
use std::io::Read;
use std::collections::HashMap;

use rusoto_core::Region;
use rusoto_credential::StaticProvider;

use rusoto_core::reactor::RequestDispatcher;
use rusoto_s3::{S3, S3Client, ListObjectsV2Request};

#[derive(Debug, Deserialize)]
pub struct Endpoint {
    pub public_url: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub s3_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Inventory {
    pub name: String,
    pub version: Option<String>,
}

impl PartialEq for Inventory {
    fn eq(&self, other: &Inventory) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Deserialize)]
pub struct Architecture {
    pub prefix: String,
    pub objects: Vec<Inventory>,
}

impl PartialEq for Architecture {
    fn eq(&self, other: &Architecture) -> bool {
        self.prefix == other.prefix
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Endpoint>, Box<Error>> {
    let mut fh = File::open(path.as_ref())?;
    let mut buffer = String::new();
    fh.read_to_string(&mut buffer)?;
    let decoded: HashMap<String, Endpoint> = toml::from_str(buffer.as_str())?;
    return Ok(decoded);
}

pub fn process_s3(endpoint: &Endpoint) -> Result<Vec<Architecture>, Box<Error>> {
	let s3_endpoint = match &endpoint.s3_endpoint {
		Some(s) => s.to_string(),
		None => {
			return Err(From::from("s3_endpoint missing"));
		}
	};
	let s3_bucket = match &endpoint.s3_bucket {
		Some(s) => s.to_string(),
		None => {
			return Err(From::from("s3_bucket missing"));
		}
	};
    let s3_key = match &endpoint.s3_key {
        Some(s) => s.to_string(),
        None => {
            return Err(From::from("s3_key missing"));
        }
    };
    let s3_secret = match &endpoint.s3_secret {
        Some(s) => s.to_string(),
        None => {
            return Err(From::from("s3_secret missing"));
        }
    };
    let provider: StaticProvider = StaticProvider::new_minimal(s3_key, s3_secret);
    let region = Region::Custom {
        name: "us-east-1".to_string(),
        endpoint: s3_endpoint,
    };
    let s3_client = S3Client::new(RequestDispatcher::default(), provider, region);
    let mut prefix_request = ListObjectsV2Request { bucket: s3_bucket.clone(), ..Default::default() };
    let mut architectures: Vec<Architecture> = Vec::new();
    loop {
        let result = s3_client.list_objects_v2(&prefix_request).sync()?;
        if !result.contents.is_some() {
            break;
        }

        let objs = &result.contents.unwrap();
        let last_obj = &objs[&objs.len() - 1];
        prefix_request.start_after = last_obj.key.clone();
        for obj in objs {
            if obj.key.is_some() {
                let key = obj.clone().key.unwrap();
                if key.ends_with("/") {
                    let arch = Architecture { prefix: key, objects: Vec::new() };
                    if !architectures.contains(&arch) {
                        architectures.push(arch);
                    }
                }
            }
            else {
                println!("got a borked S3 obj: {:?}", obj);
            }
        }
    }

    println!("Architectures: {:?}", architectures);

    for arch in architectures {
        let mut request = ListObjectsV2Request { bucket: s3_bucket.clone(), prefix: Some(arch.prefix),
            ..Default::default() };
        loop {
            let result = s3_client.list_objects_v2(&request).sync()?;
            if !result.contents.is_some() {
                break;
            }

            let objs = &result.contents.unwrap();
            let last_obj = &objs[&objs.len() - 1];
            request.start_after = last_obj.key.clone();
            for obj in objs {
                if obj.key.is_some() {
                    let key = obj.clone().key.unwrap();
                    let inventory = Inventory { name: key, version: None };
                    if !arch.objects.contains(&inventory) {
                        arch.objects.push(inventory);
                    }
                } else {
                    println!("got a borked S3 obj: {:?}", obj);
                }
            }
        }
    }
	return Ok(architectures)
}
