use std::fmt::Debug;
use std::path::Path;

use anyhow::{Context, Result};
use log::info;

use tempfile::TempDir;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::glob;

use crate::config::{relative_to_config_path, Config};

pub fn copy_from_image<P>(
    config_path: P,
    config: &Config,
    slug: &str,
    temp_dir: TempDir,
) -> Result<()>
where
    P: AsRef<Path> + Debug,
{
    let relative = Path::new(&config.copy.dst).join(&slug);
    let destination = relative_to_config_path(config_path, relative)?;
    std::fs::create_dir_all(&destination).context("Failed to create destination directory")?;

    let mut entries = Vec::new();

    let temp_dir_str = temp_dir.path().to_string_lossy();

    for src_glob in &config.copy.src {
        let abs_glob = format!("{}{}", temp_dir_str, src_glob);
        for glob_result in
            glob(&abs_glob).context(format!("Failed to read glob pattern {}", abs_glob))?
        {
            match glob_result {
                Ok(path) => entries.push(path),
                Err(e) => info!("{:?}", e),
            }
        }
    }

    copy_items(&entries, &destination, &CopyOptions::new())
        .context("Failed to copy image contents")?;

    Ok(())
}
