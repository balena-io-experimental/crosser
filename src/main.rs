use anyhow::{bail, Result};

mod cloud;
mod config;
mod tar;

fn main() -> Result<()> {
    let app = "crosser";

    let crosser = config::read_config();

    let applications = crate::cloud::get_application_by_name(&crosser.token, app)?;

    if applications.len() == 0 {
        bail!("Application not found");
    }

    let gzip = crate::tar::tar_gz_dockerfile_directory("./app")?;

    let username = crate::cloud::get_application_username(&crosser.token, app)?;

    println!("Application username: {}", username);

    crate::cloud::build_application(&crosser.token, &username, app, gzip)?;

    Ok(())
}
