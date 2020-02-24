use std::fs::File;

use anyhow::{Context, Result};

use ron::de::from_reader;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
    pub name: String,
    pub src: String,
    pub dst: Vec<(String, String)>,
    pub platforms: Vec<(Arch, Libc)>,
}

#[derive(Debug, Deserialize)]
pub enum Arch {
    Aarch64,
    Armv7hf,
}

#[derive(Debug, Deserialize)]
pub enum Libc {
    Glibc,
    Musl,
}

pub fn read_config(cli_args: &crate::cli::CliArgs) -> Result<Config> {
    let config_file = if let Some(ref config_file) = cli_args.config {
        config_file
    } else {
        "crosser.ron"
    };

    let file =
        File::open(config_file).context(format!("Opening config file '{}' failed", config_file))?;

    Ok(from_reader(file).context(format!(
        "Deserializing config file '{}' failed",
        config_file
    ))?)
}
