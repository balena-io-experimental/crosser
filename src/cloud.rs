use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

const API_BASE: &str = "https://api.balena-cloud.com";
const BUILDER_BASE: &str = "https://builder.balena-cloud.com";

const ENDPOINT_APPLICATION: &str = "v5/application";
const BUILD_ENDPOINT: &str = "v3/build";

#[derive(Debug, Deserialize)]
struct Response<T> {
    #[serde(rename = "d")]
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Application {
    id: u64,
    app_name: String,
    device_type: String,
}

fn get_application_by_name_endpoint(app: &str) -> String {
    format!("{}?$filter=app_name eq '{}'", ENDPOINT_APPLICATION, app)
}

fn get(token: &str, endpoint: &str) -> Result<reqwest::blocking::Response> {
    let url = format_url(endpoint);
    Ok(reqwest::blocking::Client::new()
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()?)
}

fn format_url(endpoint: &str) -> String {
    format!("{}/{}", API_BASE, endpoint)
}

pub fn get_application_by_name(token: &str, app: &str) -> Result<Vec<Application>> {
    Ok(get(token, &get_application_by_name_endpoint(app))?
        .json::<Response<Application>>()?
        .data)
}

pub fn get_application_username(token: &str, app: &str) -> Result<String> {
    let mut users = get(token, &get_application_username_endpoint(app))?
        .json::<Response<Value>>()?
        .data;

    Ok(Value::to_string(
        users
            .pop()
            .context("One application owner expected")?
            .pointer("/user/0/username")
            .context("Username pointer failed")?,
    ))
}

fn get_application_username_endpoint(app: &str) -> String {
    format!(
        "{}?$expand=user($select=username)&$filter=app_name eq '{}'&$select=id",
        ENDPOINT_APPLICATION, app
    )
}

pub fn build_application(token: &str, username: &str, app: &str, gzip: Vec<u8>) -> Result<()> {
    let endpoint = get_build_application_endpoint(username, app);
    let url = format!("{}/{}", BUILDER_BASE, endpoint);
    let response = reqwest::blocking::Client::new()
        .post(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .header(reqwest::header::CONTENT_ENCODING, "gzip")
        .body(gzip)
        .send()?;

    println!("{:?}", response);

    Ok(())
}

fn get_build_application_endpoint(username: &str, app: &str) -> String {
    format!(
        "{}?owner={}&app={}&dockerfilePath=&emulated=false&nocache=false&headless=false",
        BUILD_ENDPOINT, username, app
    )
}
