use std::fs::File;

use anyhow::{Context, Result};

use ron::de::from_reader;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
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

pub fn read_config(cli_args: &crate::cli::CliArgs) -> Result<Config> {
    let file = File::open(&cli_args.config)
        .context(format!("Opening config file '{}' failed", cli_args.config))?;

    Ok(from_reader(file).context(format!(
        "Deserializing config file '{}' failed",
        cli_args.config
    ))?)
}
