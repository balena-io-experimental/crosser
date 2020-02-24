use anyhow::Result;

mod cli;
mod cloud;
mod registry;
mod tar;

#[tokio::main]
async fn main() -> Result<()> {
    let app_name = "crosser";

    let crosser = cli::read_cli_args();

    let application = crate::cloud::get_application_by_name(&crosser.token, app_name).await?;

    let user = crate::cloud::get_application_user(&crosser.token, &application).await?;

    println!("Application user: {:?}", user);

    let registration = crate::cloud::register_device(&crosser.token, &application, &user).await?;

    println!("Registered device: {:?}", registration);

    let gzip = crate::tar::tar_gz_dockerfile_directory("./app")?;

    let success =
        crate::cloud::build_application(&crosser.token, &application, &user, gzip).await?;

    println!("Build result: {:?}", success);

    let image_url = crate::cloud::get_device_image_url(&crosser.token, &registration.uuid).await?;

    println!("Image URL: {}", image_url);

    let _ = crate::registry::download_image(&image_url, &registration).await?;

    Ok(())
}
