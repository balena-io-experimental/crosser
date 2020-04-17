use std::fmt::Debug;
use std::fs::File;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use serde_yaml::from_reader;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub copy: CopySpec,
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
pub struct CopySpec {
    pub src: Vec<String>,
    pub dst: String,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub slug: String,
    pub device_type: String,
    pub source: String,
}

pub fn read_config(path: &str) -> Result<Config> {
    let file = File::open(path)
        .context(format!("Opening config file '{}' failed", path))?;

    Ok(from_reader(file).context(format!(
        "Deserializing config file '{}' failed",
        path
    ))?)
}

pub fn relative_to_config_path<P, Q>(config_path: P, path: Q) -> Result<PathBuf>
where
    P: AsRef<Path> + Debug, Q: AsRef<Path> + Debug {
    let mut absolute = current_dir()?;
    absolute.push(config_path);
    absolute.pop();
    absolute.push(path);
    Ok(absolute)
}
