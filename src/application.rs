use anyhow::{Context, Result};
use log::info;

use serde::Deserialize;

use crate::api::{get, Response};

const ENDPOINT_APPLICATION: &str = "v5/application";

#[derive(Debug, Deserialize)]
pub struct Application {
    pub id: u64,
    #[serde(rename = "app_name")]
    pub name: String,
    pub device_type: String,
}

#[derive(Debug, Deserialize)]
struct ApplicationUsers {
    id: u64,
    user: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
}

fn get_application_by_name_endpoint(app: &str) -> String {
    format!("{}?$filter=app_name eq '{}'", ENDPOINT_APPLICATION, app)
}

pub async fn get_application_by_name(token: &str, app: &str) -> Result<Application> {
    info!("Getting application by name '{}'", app);
    Ok(get(token, &get_application_by_name_endpoint(app))
        .await?
        .json::<Response<Application>>()
        .await?
        .data
        .pop()
        .context("Application not found")?)
}

pub async fn get_application_user(token: &str, application: &Application) -> Result<User> {
    info!("Getting application user '{}'", application.name);
    let mut response = get(token, &get_application_user_endpoint(&application.name))
        .await?
        .json::<Response<ApplicationUsers>>()
        .await?
        .data;

    Ok(response
        .pop()
        .context("Application not found")?
        .user
        .pop()
        .context("No application users defined")?)
}

fn get_application_user_endpoint(app: &str) -> String {
    format!(
        "{}?$expand=user($select=id,username)&$filter=app_name eq '{}'&$select=id",
        ENDPOINT_APPLICATION, app
    )
}
