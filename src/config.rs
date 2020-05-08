use std::env::current_dir;
use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use serde_yaml::from_reader;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub source: String,
    pub copy: CopySpec,
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
pub struct CopySpec {
    pub from_image: Vec<String>,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub slug: String,
    pub device_type: String,
    pub dockerfile: String,
}

pub fn read_config(path: &str) -> Result<Config> {
    let file = File::open(path).context(format!("Opening config file '{}' failed", path))?;

    Ok(from_reader(file).context(format!("Deserializing config file '{}' failed", path))?)
}

pub fn config_dir<P>(config_path: P) -> Result<PathBuf>
where
    P: AsRef<Path> + Debug,
{
    let mut absolute = current_dir()?;
    absolute.push(config_path);
    absolute.pop();
    Ok(absolute)
}

pub fn config_name<P>(config_path: P) -> Result<String>
where
    P: AsRef<Path> + Debug,
{
    Ok(config_path
        .as_ref()
        .file_stem()
        .context(format!(
            "Cannot extract file stem from config path {:?}",
            config_path
        ))?
        .to_string_lossy()
        .to_string())
}
