use anyhow::Result;
use log::info;

mod api;
mod application;
mod builder;
mod cli;
mod config;
mod device;
mod logger;
mod registry;
mod tar;

#[tokio::main]
async fn main() -> Result<()> {
    crate::logger::init()?;

    let cli_args = cli::read_cli_args();

    let config = config::read_config(&cli_args)?;

    if false {
        for (arch, libc) in &config.platforms {
            info!("{} / {}", arch, libc);
        }
        info!("Config: {:?}", config);
        std::process::exit(0);
    }

    let application =
        crate::application::get_application_by_name(&config.token, &config.name).await?;

    let user = crate::application::get_application_user(&config.token, &application).await?;

    info!("Application user: {:?}", user);

    let registration = crate::device::register_device(&config.token, &application, &user).await?;

    info!("Registered device: {:?}", registration);

    let gzip = crate::tar::tar_gz_dockerfile_directory(&config.src)?;

    let success =
        crate::builder::build_application(&config.token, &application, &user, gzip).await?;

    info!("Build result: {:?}", success);

    let image_url = crate::device::get_device_image_url(&config.token, &registration.uuid).await?;

    info!("Image URL: {}", image_url);

    let _ = crate::registry::download_image(&image_url, &registration).await?;

    Ok(())
}
