use std::path::Path;

use anyhow::{Context, Result};
use log::info;

use tempfile::TempDir;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::glob;

use crate::config::Config;

pub fn copy_from_image(config: &Config, slug: &str, temp_dir: TempDir) -> Result<()> {
    let destination = Path::new(&config.copy.dst).join(&slug);
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
