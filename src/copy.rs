use std::fmt::Debug;
use std::path::Path;

use anyhow::{Context, Result};
use log::info;

use tempfile::TempDir;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::glob;
use ignore::Walk;

use crate::config::{Config, Target};

pub fn copy_from_image(config: &Config, slug: &str, temp_dir: TempDir) -> Result<()> {
    let relative = Path::new(&config.copy.to).join(&slug);
    std::fs::create_dir_all(&relative).context("Failed to create destination directory")?;

    let mut entries = Vec::new();

    let temp_dir_str = temp_dir.path().to_string_lossy();

    for src_glob in &config.copy.from_image {
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

    copy_items(&entries, &relative, &CopyOptions::new())
        .context("Failed to copy image contents")?;

    Ok(())
}

pub fn assemble_sources<P>(config_dir: P, config: &Config, target: &Target) -> Result<TempDir>
where
    P: AsRef<Path> + Debug,
{
    info!("Assembling sources");

    let temp_dir =
        TempDir::new().context("Creating temp directory for assembling sources failed")?;

    let dockerfile_from = config_dir.as_ref().join(&target.dockerfile);
    let dockerfile_to = temp_dir.path().join(
        dockerfile_from
            .file_name()
            .context("Failed to get Dockerfile file name")?,
    );

    std::fs::copy(dockerfile_from, dockerfile_to).context("Failed to copy Dockerfile")?;

    std::env::set_current_dir(config_dir.as_ref().join(&config.source))?;

    for result in Walk::new("./") {
        if let Ok(entry) = result {
            if let Some(file_type) = entry.file_type() {
                let destination = temp_dir.path().join(entry.path());

                if file_type.is_dir() {
                    std::fs::create_dir_all(&destination).context(format!(
                        "Failed to create destination directory {:?}",
                        destination
                    ))?;
                } else {
                    std::fs::copy(entry.path(), &destination).context(format!(
                        "Failed to copy to destination {:?} {:?}",
                        entry, destination
                    ))?;
                }
            }
        }
    }

    Ok(temp_dir)
}
