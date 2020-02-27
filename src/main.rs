mod api;
mod application;
mod builder;
mod cli;
mod config;
mod device;
mod logger;
mod registry;
mod state;
mod tar;

use anyhow::{Context, Result};
use log::info;

use fs_extra::dir::{copy, CopyOptions};

use crate::application::{get_application_user, get_or_create_application, Application, User};
use crate::builder::build_application;
use crate::cli::{read_cli_args, CliArgs};
use crate::config::read_config;
use crate::device::{get_device_image_url, register_device, DeviceRegistration};
use crate::registry::download_image;
use crate::state::{add_device_registration, get_device_registration, load_state, State};
use crate::tar::tar_gz_dockerfile_directory;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    let cli_args = read_cli_args();

    let config = read_config(&cli_args)?;

    let mut state = load_state(&cli_args)?;

    for target in &config.targets {
        info!(
            "Building '{}' for '{}' from '{}'",
            target.slug, target.device_type, target.source
        );

        let application_name = format!("{}-{}", config.name, target.slug);

        let application =
            get_or_create_application(&config.token, &application_name, &target.device_type)
                .await?;

        let user = get_application_user(&config.token, &application).await?;

        let registration = get_or_register_device(
            &config.token,
            &cli_args,
            &target.slug,
            &application,
            &user,
            &mut state,
        )
        .await?;

        let gzip = tar_gz_dockerfile_directory(&target.source)?;

        build_application(&config.token, &application, &user, gzip).await?;

        let image_url = get_device_image_url(&config.token, &registration.uuid).await?;

        let temp_dir = download_image(&image_url, &registration).await?;

        for copy_spec in &config.copy {
            let source = temp_dir.path().join(&copy_spec.from[1..]);
            let destination = std::path::Path::new(&copy_spec.to).join(&target.slug);
            std::fs::create_dir_all(&destination)
                .context("Failed to create destination directory")?;
            info!("Copy from {:?} to {:?}", source, destination);
            copy(source, destination, &CopyOptions::new())
                .context("Failed to copy image contents")?;
        }
    }

    Ok(())
}

async fn get_or_register_device(
    token: &str,
    cli_args: &CliArgs,
    slug: &str,
    application: &Application,
    user: &User,
    state: &mut State,
) -> Result<DeviceRegistration> {
    Ok(
        if let Some(registration) = get_device_registration(state, slug) {
            info!(
                "Reusing device '{}' ({})",
                registration.uuid, registration.id
            );

            registration
        } else {
            let registration = register_device(token, &application, &user).await?;

            add_device_registration(&cli_args, slug, &registration, state)?;

            registration
        },
    )
}
