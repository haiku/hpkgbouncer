
use toml;
use std::fs::File;
use std::path::{Path,PathBuf};
use std::error::Error;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct EndpointConfig {
    pub public_url: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_key: Option<String>,
    pub s3_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Endpoints {
    pub name: String,
    pub config: EndpointConfig,
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Endpoints, Box<Error>> {
    let mut fh = File::open(path.as_ref())?;
    let mut buffer = String::new();
    fh.read_to_string(&mut buffer)?;
    let decoded: Endpoints = toml::from_str(buffer.as_str())?;
    return Ok(decoded);
}
