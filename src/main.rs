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

use anyhow::Result;
use log::info;

use crate::application::{get_application_user, get_or_create_application};
use crate::builder::build_application;
use crate::device::{get_device_image_url, register_device};
use crate::registry::download_image;
use crate::state::{add_device_registration, get_device_registration, load_state};
use crate::tar::tar_gz_dockerfile_directory;

#[tokio::main]
async fn main() -> Result<()> {
    crate::logger::init()?;

    let cli_args = cli::read_cli_args();

    let config = config::read_config(&cli_args)?;

    let mut state = load_state(&cli_args)?;

    for target in &config.targets {
        info!(
            "Building '{}' for '{}' from {}",
            target.slug, target.device_type, target.source
        );

        let application_name = format!("{}-{}", config.name, target.slug);

        let application =
            get_or_create_application(&config.token, &application_name, &target.device_type)
                .await?;

        let user = get_application_user(&config.token, &application).await?;

        info!("Application user: {:?}", user);

        let registration = if let Some(registration) = get_device_registration(&state, &target.slug)
        {
            info!("Reusing device: {:?}", registration);

            registration
        } else {
            let registration = register_device(&config.token, &application, &user).await?;

            add_device_registration(&cli_args, &mut state, &target.slug, &registration)?;

            info!("Registered device: {:?}", registration);

            registration
        };

        let gzip = tar_gz_dockerfile_directory(&target.source)?;

        let success = build_application(&config.token, &application, &user, gzip).await?;

        info!("Build result: {:?}", success);

        let image_url = get_device_image_url(&config.token, &registration.uuid).await?;

        info!("Image URL: {}", image_url);

        download_image(&image_url, &registration).await?;
    }

    Ok(())
}
