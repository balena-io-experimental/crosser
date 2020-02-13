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

    let mut emcoder = GzEncoder::new(Vec::new(), Compression::default());
    emcoder.write_all(&data)?;
    let compressed_bytes = emcoder.finish()?;

    Ok(compressed_bytes)
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
