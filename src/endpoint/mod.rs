
use toml;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::Read;
use std::collections::HashMap;
use std::clone::Clone;

use s3::bucket::Bucket;
use s3::region::Region;
use s3::credentials::Credentials;
use s3::error::S3Error;

use regex::Regex;

#[derive(Debug, Deserialize, Clone)]
pub struct Endpoint {
    pub public_url: String,
    pub bucket_prefix: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub s3_secret: Option<String>,
    pub s3_region: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Inventory {
    pub name: String,
    pub version: Option<String>,
}

impl PartialEq for Inventory {
    fn eq(&self, other: &Inventory) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Architecture {
    pub prefix: String,
    pub latest: Option<Inventory>,
    pub objects: Vec<Inventory>,
}

impl PartialEq for Architecture {
    fn eq(&self, other: &Architecture) -> bool {
        self.prefix == other.prefix
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Endpoint>, Box<dyn Error>> {
    let mut fh = File::open(path.as_ref())?;
    let mut buffer = String::new();
    fh.read_to_string(&mut buffer)?;
    let decoded: HashMap<String, Endpoint> = toml::from_str(buffer.as_str())?;
    return Ok(decoded);
}

pub fn process_s3(endpoint: &Endpoint) -> Result<Vec<Architecture>, Box<dyn Error>> {
    let bucket_prefix = match &endpoint.bucket_prefix {
        Some(s) => s.to_string(),
        None => "/".to_string(),
    };
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
    // Set common default of us-east-1
    let s3_region = match &endpoint.s3_region {
        Some(s) => s.to_string(),
        None => "us-east-1".to_string(),
    };

    let region = Region::Custom { region: s3_region, endpoint: s3_endpoint };
    let credentials = Credentials::new(Some(s3_key), Some(s3_secret), None, None);
    let bucket = Bucket::new(&s3_bucket, region, credentials)?;

    let mut architectures: Vec<Architecture> = Vec::new();
    println!("LISTALL!");
    let results = bucket.list_all(bucket_prefix, Some("/".to_string())).unwrap();
    println!("LISTALL!");
    for (list, code) in results {
        println!("{:?}", list);
    }
    //loop {
    //    let result = s3_client.list_objects_v2(&prefix_request).sync()?;
    //    if !result.contents.is_some() {
    //        break;
    //    }

    //    let objs = &result.contents.unwrap();
    //    let last_obj = &objs[&objs.len() - 1];
    //    prefix_request.start_after = last_obj.key.clone();
    //    for obj in objs {
    //        if obj.key.is_some() {
    //            let key = obj.clone().key.unwrap();
    //            if key.ends_with("/") {
    //                let arch = Architecture { prefix: key, latest: None, objects: Vec::new() };
    //                if !architectures.contains(&arch) {
    //                    architectures.push(arch);
    //                }
    //            }
    //        }
    //        else {
    //            println!("got a borked S3 obj: {:?}", obj);
    //        }
    //    }
    //}

    //let file_re = Regex::new(r"^.*(hrev\d+).*\.zip$")?;
    //let path_re = Regex::new(r"^.*(hrev\d+)/$")?;
    //let hrev_re = Regex::new(r"hrev(\d+)")?;

    //for ref mut arch in &mut architectures {
    //    let mut request = ListObjectsV2Request { bucket: s3_bucket.clone(), prefix: Some(arch.prefix.clone()),
    //        ..Default::default() };
    //    let mut latest = 0;
    //    loop {
    //        let result = s3_client.list_objects_v2(&request).sync()?;
    //        if !result.contents.is_some() {
    //            break;
    //        }
    //        let objs = &result.contents.unwrap();
    //        let last_obj = &objs[&objs.len() - 1];
    //        request.start_after = last_obj.key.clone();
    //        for obj in objs {
    //            if obj.key.is_some() {
    //                let key = obj.clone().key.unwrap();
    //                let file = key.trim_left_matches(&arch.prefix);
    //                let mut inventory = Inventory { name: file.clone().to_string(), version: None };
    //                if file_re.is_match(file) {
    //                    let caps = file_re.captures(file).unwrap();
    //                    inventory.version = Some(caps[1].to_string());
    //                    arch.objects.push(inventory.clone());
    //                } else if path_re.is_match(file) {
    //                    let caps = path_re.captures(file).unwrap();
    //                    inventory.version = Some(caps[1].to_string());
    //                    arch.objects.push(inventory.clone());
    //                } else {
    //                    continue
    //                }
    //                let version = inventory.version.clone().unwrap();
    //                let hrev_string: String = hrev_re.captures(&version).unwrap()[1].to_string();
    //                let hrev: i32 = hrev_string.parse()?;
    //                if hrev > latest {
    //                    latest = hrev;
    //                    arch.latest = Some(inventory);
    //                }
    //            } else {
    //                println!("got a borked S3 obj: {:?}", obj);
    //            }
    //        }
    //    }
    //}
    return Ok(architectures)
}
