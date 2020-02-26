mod api;
mod application;
mod builder;
mod cli;
mod config;
mod device;
mod logger;
mod registry;
mod tar;

use anyhow::Result;
use log::info;

use crate::application::get_or_create_application;

#[tokio::main]
async fn main() -> Result<()> {
    crate::logger::init()?;

    let cli_args = cli::read_cli_args();

    let config = config::read_config(&cli_args)?;

    for target in &config.targets {
        info!(
            "Building '{}' for '{}' from {}",
            target.slug, target.device_type, target.source
        );

        let application_name = format!("{}-{}", config.name, target.slug);

        let application =
            get_or_create_application(&config.token, &application_name, &target.device_type)
                .await?;

        let user = crate::application::get_application_user(&config.token, &application).await?;

        info!("Application user: {:?}", user);

        let registration =
            crate::device::register_device(&config.token, &application, &user).await?;

        info!("Registered device: {:?}", registration);

        let gzip = crate::tar::tar_gz_dockerfile_directory(&target.source)?;

        let success =
            crate::builder::build_application(&config.token, &application, &user, gzip).await?;

        info!("Build result: {:?}", success);

        let image_url =
            crate::device::get_device_image_url(&config.token, &registration.uuid).await?;

        info!("Image URL: {}", image_url);

        crate::registry::download_image(&image_url, &registration).await?;
    }

    Ok(())
}
