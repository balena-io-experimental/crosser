use anyhow::{bail, Result};

mod cloud;
mod config;
mod tar;

#[tokio::main]
async fn main() -> Result<()> {
    let app = "crosser";

    let crosser = config::read_config();

    let applications = crate::cloud::get_application_by_name(&crosser.token, app).await?;

    if applications.is_empty() {
        bail!("Application not found");
    }

    let gzip = crate::tar::tar_gz_dockerfile_directory("./app")?;

    let username = crate::cloud::get_application_username(&crosser.token, app).await?;

    println!("Application username: {}", username);

    crate::cloud::build_application(&crosser.token, &username, app, gzip).await?;

    Ok(())
}
