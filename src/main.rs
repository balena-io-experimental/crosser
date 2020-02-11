use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

mod args;

const API_BASE: &str = "https://api.balena-cloud.com";

const ENDPOINT_APPLICATION: &str = "v5/application";
const ENDPOINT_USER: &str = "v5/user";

#[derive(Debug, Serialize, Deserialize)]
struct Application {
    id: u64,
    app_name: String,
    device_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    actor: u64,
    username: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response<T> {
    #[serde(rename = "d")]
    data: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApplicationResponse(HashMap<String, Vec<Application>>);

fn get_application_by_name_endpoint(name: &str) -> String {
    format!("{}?$filter=app_name eq '{}'", ENDPOINT_APPLICATION, name)
}

fn get(token: &str, endpoint: &str) -> Result<reqwest::blocking::Response> {
    let url = format_url(endpoint);
    println!("get {}", url);
    Ok(reqwest::blocking::Client::new()
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()?)
}

fn format_url(endpoint: &str) -> String {
    format!("{}/{}", API_BASE, endpoint)
}

fn get_application_by_name(token: &str, name: &str) -> Result<Vec<Application>> {
    Ok(get(token, &get_application_by_name_endpoint(name))?
        .json::<Response<Application>>()?
        .data)
}

fn get_applications(token: &str) -> Result<Vec<Application>> {
    Ok(get(token, ENDPOINT_APPLICATION)?
        .json::<Response<Application>>()?
        .data)
}

fn get_users(token: &str) -> Result<Vec<User>> {
    Ok(get(token, ENDPOINT_USER)?.json::<Response<User>>()?.data)
}

fn main() -> Result<()> {
    let args = args::get_cli_args();

    for user in get_users(&args.token)? {
        println!("{}", user.username);
    }

    for app in get_application_by_name(&args.token, "crosser")? {
        println!("{} [{}]", app.app_name, app.device_type);
    }

    for app in get_applications(&args.token)? {
        println!("{} [{}]", app.app_name, app.device_type);
    }

    Ok(())
}
