use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{get, post};
use crate::application::{Application, User};

const REGISTER_ENDPOINT: &str = "device/register";

#[derive(Debug, Serialize)]
pub struct DeviceRegistrationRequest {
    #[serde(rename = "application")]
    application_id: u64,
    #[serde(rename = "user")]
    user_id: u64,
    device_type: String,
    uuid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
