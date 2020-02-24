use anyhow::Result;

mod cli;
mod cloud;
mod config;
mod registry;
mod tar;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = cli::read_cli_args();

    let config = config::read_config(&cli_args)?;

    if true {
        for (arch, libc) in &config.platforms {
            println!("{} / {}", arch, libc);
        }
        println!("Config: {:?}", config);
        std::process::exit(0);
    }

    let application = crate::cloud::get_application_by_name(&config.token, &config.name).await?;

    let user = crate::cloud::get_application_user(&config.token, &application).await?;

    println!("Application user: {:?}", user);

    let registration = crate::cloud::register_device(&config.token, &application, &user).await?;

    println!("Registered device: {:?}", registration);

    let gzip = crate::tar::tar_gz_dockerfile_directory("./app")?;

    let success = crate::cloud::build_application(&config.token, &application, &user, gzip).await?;

    println!("Build result: {:?}", success);

    let image_url = crate::cloud::get_device_image_url(&config.token, &registration.uuid).await?;

    println!("Image URL: {}", image_url);

    let _ = crate::registry::download_image(&image_url, &registration).await?;

    Ok(())
}
