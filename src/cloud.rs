use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const API_BASE: &str = "https://api.balena-cloud.com";

const ENDPOINT_APPLICATION: &str = "v5/application";
const REGISTER_ENDPOINT: &str = "device/register";

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

async fn get(token: &str, endpoint: &str) -> Result<reqwest::Response> {
    let url = format!("{}/{}", API_BASE, endpoint);
    Ok(reqwest::Client::new()
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?)
}

async fn post<T: Serialize + ?Sized>(
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

#[derive(Debug, Serialize)]
pub struct DeviceRegistrationRequest {
    #[serde(rename = "application")]
    application_id: u64,
    #[serde(rename = "user")]
    user_id: u64,
    device_type: String,
    uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceRegistration {
    pub id: u64,
    pub uuid: String,
    pub api_key: String,
}

pub async fn register_device(
    token: &str,
    application: &Application,
    user: &User,
) -> Result<DeviceRegistration> {
    let input = DeviceRegistrationRequest {
        application_id: application.id,
        user_id: user.id,
        device_type: application.device_type.clone(),
        uuid: new_uuid()?,
    };

    Ok(post(token, REGISTER_ENDPOINT, &input)
        .await?
        .json::<DeviceRegistration>()
        .await?)
}

fn new_uuid() -> Result<String> {
    let mut buf = [0; 16];
    getrandom::getrandom(&mut buf).context("Random generation failed")?;
    Ok(hex::encode(buf))
}

pub async fn get_device_image_url(token: &str, uuid: &str) -> Result<String> {
    let value = get(token, &get_device_state_endpoint(uuid))
        .await?
        .json::<Value>()
        .await?;

    Ok(get_image_from_device_state(&value).context("Image not found in device state")?)
}

fn get_device_state_endpoint(uuid: &str) -> String {
    format!("device/v2/{}/state", uuid)
}

fn get_image_from_device_state(val: &Value) -> Option<String> {
    if let Some(map) = val.as_object() {
        for (key, inner) in map.iter() {
            if key == "image" {
                return inner.as_str().map(|s| s.to_string());
            } else {
                let result = get_image_from_device_state(inner);
                if result.is_some() {
                    return result;
                }
            }
        }
    }

    None
}
