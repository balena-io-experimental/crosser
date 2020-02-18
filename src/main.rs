use anyhow::Result;

mod cloud;
mod config;
mod tar;

#[tokio::main]
async fn main() -> Result<()> {
    let app_name = "crosser";

    let crosser = config::read_config();

    let application = crate::cloud::get_application_by_name(&crosser.token, app_name).await?;

    let user = crate::cloud::get_application_user(&crosser.token, &application).await?;

    println!("Application user: {:?}", user);

    let registration = crate::cloud::register_device(&crosser.token, &application, &user).await?;

    println!("Registered device: {:?}", registration);

    let gzip = crate::tar::tar_gz_dockerfile_directory("./app")?;

    let success =
        crate::cloud::build_application(&crosser.token, &application, &user, gzip).await?;

    println!("Build result: {:?}", success);

    let image = crate::cloud::get_device_image(&crosser.token, &registration.uuid).await?;

    println!("Image: {}", image);

    Ok(())
}
