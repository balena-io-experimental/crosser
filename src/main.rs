#[macro_use]
extern crate clap;

mod api;
mod application;
mod builder;
mod cli;
mod config;
mod copy;
mod device;
mod logger;
mod registry;
mod tar;
mod variable;

use anyhow::Result;
use log::info;

use crate::application::{get_application_user, get_or_create_application, Application, User};
use crate::builder::build_application;
use crate::cli::read_cli_args;
use crate::config::{relative_to_config_path, read_config};
use crate::copy::copy_from_image;
use crate::device::{
    create_device, get_device_image_url, get_device_registration, DeviceRegistration,
};
use crate::registry::download_image;
use crate::tar::tar_gz_dockerfile_directory;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    let cli_args = read_cli_args();

    let config = read_config(&cli_args.config)?;

    for target in &config.targets {
        let source = relative_to_config_path(&cli_args.config, &target.source)?;

        info!(
            "Building '{}' for '{}' from '{}'",
            target.slug, target.device_type, source.to_string_lossy()
        );

        let application_name = format!("{}-{}", config.name, target.slug);

        let application =
            get_or_create_application(&cli_args.token, &application_name, &target.device_type)
                .await?;

        let user = get_application_user(&cli_args.token, &application).await?;

        let registration =
            get_or_create_device(&cli_args.token, &application, &target.slug, &user).await?;

        let gzip = tar_gz_dockerfile_directory(&source)?;

        build_application(&cli_args.token, &application, &user, gzip).await?;

        let image_url = get_device_image_url(&cli_args.token, &registration.uuid).await?;

        let temp_dir = download_image(&image_url, &registration).await?;

        copy_from_image(&cli_args.config, &config, &target.slug, temp_dir)?;
    }

    Ok(())
}

async fn get_or_create_device(
    token: &str,
    application: &Application,
    slug: &str,
    user: &User,
) -> Result<DeviceRegistration> {
    Ok(
        if let Some(registration) = get_device_registration(token, application, slug).await? {
            info!(
                "Reusing device '{}' ({})",
                registration.uuid, registration.id
            );

            registration
        } else {
            create_device(token, &application, &user, slug).await?
        }
    )
}
