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
    crate::logger::init()?;

    let cli_args = read_cli_args();

    let config = read_config(&cli_args)?;

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

        let success = build_application(&config.token, &application, &user, gzip).await?;

        info!("Build result: {:?}", success);

        let image_url = get_device_image_url(&config.token, &registration.uuid).await?;

        info!("Image URL: {}", image_url);

        download_image(&image_url, &registration).await?;
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
            info!("Reusing device: {:?}", registration);

            registration
        } else {
            let registration = register_device(token, &application, &user).await?;

            add_device_registration(&cli_args, slug, &registration, state)?;

            info!("Registered device: {:?}", registration);

            registration
        },
    )
}
