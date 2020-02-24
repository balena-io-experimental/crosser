use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.balena-cloud.com";

const ENDPOINT_APPLICATION: &str = "v5/application";

#[derive(Debug, Deserialize)]
struct Response<T> {
    #[serde(rename = "d")]
    pub data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Application {
    pub id: u64,
    #[serde(rename = "app_name")]
    pub name: String,
    pub device_type: String,
}

fn get_application_by_name_endpoint(app: &str) -> String {
    format!("{}?$filter=app_name eq '{}'", ENDPOINT_APPLICATION, app)
}

pub async fn get(token: &str, endpoint: &str) -> Result<reqwest::Response> {
    let url = format!("{}/{}", API_BASE, endpoint);
    Ok(reqwest::Client::new()
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?)
}

pub async fn post<T: Serialize + ?Sized>(
    token: &str,
    endpoint: &str,
    json: &T,
) -> Result<reqwest::Response> {
    let url = format!("{}/{}", API_BASE, endpoint);
    Ok(reqwest::Client::new()
        .post(&url)
        .json(json)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?)
}

pub async fn get_application_by_name(token: &str, app: &str) -> Result<Application> {
    Ok(get(token, &get_application_by_name_endpoint(app))
        .await?
        .json::<Response<Application>>()
        .await?
        .data
        .pop()
        .context("Application not found")?)
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

pub async fn get_application_user(token: &str, application: &Application) -> Result<User> {
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
