use std::io::prelude::*;
use std::path::Path;

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

pub fn tar_gz_dockerfile_directory<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    tar_gz_dockerfile_directory_impl(path).context("Creating tar gz stream from directory failed")
}

pub fn tar_gz_dockerfile_directory_impl<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let data = tar_dockerfile_directory(path)?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&data)?;
    let archive = encoder.finish()?;
    Ok(archive)
}

fn tar_dockerfile_directory<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let mut data: Vec<u8> = Vec::new();

    Builder::new(&mut data)
        .append_dir_all(".", path)
        .context("Creating tar stream from directory failed")?;

    Ok(data)
}
