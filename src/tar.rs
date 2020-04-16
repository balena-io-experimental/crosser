use std::fmt::Debug;
use std::io::prelude::*;
use std::path::Path;

use anyhow::{Context, Result};
use log::info;

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

pub fn tar_gz_dockerfile_directory<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path> + Debug,
{
    info!("Creating tar gz stream from {:?}", path);
    tar_gz_dockerfile_directory_impl(&path)
        .context(format!("Creating tar gz stream from {:?} failed", path))
}

fn tar_gz_dockerfile_directory_impl<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path> + Debug,
{
    let data = tar_dockerfile_directory(path)?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&data)?;
    let archive = encoder.finish()?;
    Ok(archive)
}

fn tar_dockerfile_directory<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path> + Debug,
{
    let mut data: Vec<u8> = Vec::new();

    Builder::new(&mut data)
        .append_dir_all(".", &path)
        .context(format!("Creating tar stream from {:?} failed", path))?;

    Ok(data)
}
