use anyhow::{Context, Result};
use log::info;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{get, patch, post, Response};
use crate::application::{Application, User};
use crate::variable::{get_device_api_key, store_device_api_key};

const REGISTER_ENDPOINT: &str = "device/register";
const DEVICE_ENDPOINT: &str = "v5/device";

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Device {
    pub id: u64,
    pub uuid: String,
    pub device_type: String,
    #[serde(rename = "device_name")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceNameData {
    pub device_name: String,
}

pub async fn create_device(
    token: &str,
    application: &Application,
    user: &User,
    name: &str,
) -> Result<DeviceRegistration> {
    info!("Creating device '{}'", name);

    let registration = register_device(token, application, user).await?;

    rename_device(token, &registration, name).await?;

    store_device_api_key(token, registration.id, &registration.api_key)
        .await?;

    Ok(registration)
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

    let registration = post(token, REGISTER_ENDPOINT, &input)
        .await?
        .json::<DeviceRegistration>()
        .await?;

    info!(
        "Device registered '{}' ({})",
        registration.uuid, registration.id
    );

    Ok(registration)
}

pub async fn rename_device(
    token: &str,
    registration: &DeviceRegistration,
    name: &str,
) -> Result<()> {
    let name_data = DeviceNameData {
        device_name: name.to_string(),
    };

    let _result = patch(token, &get_device_id_endpoint(registration.id), &name_data).await?;

    Ok(())
}

fn get_device_id_endpoint(device_id: u64) -> String {
    format!("{}({})", DEVICE_ENDPOINT, device_id)
}

fn new_uuid() -> Result<String> {
    let mut buf = [0; 16];
    getrandom::getrandom(&mut buf).context("Random generation failed")?;
    Ok(hex::encode(buf))
}

pub async fn get_device_image_url(token: &str, uuid: &str) -> Result<String> {
    info!("Getting image URL from '{}' device state", uuid);

    let value = get(token, &get_device_state_endpoint(uuid))
        .await?
        .json::<Value>()
        .await?;

    let image_url =
        get_image_from_device_state(&value).context("Image not found in device state")?;

    Ok(image_url)
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

pub async fn get_device_registration(
    token: &str,
    application: &Application,
    slug: &str,
) -> Result<Option<DeviceRegistration>> {
    if let Some(device) = get_device_by_name(token, application, slug).await? {
        if let Some(api_key) = get_device_api_key(token, device.id).await? {
            return Ok(Some(DeviceRegistration {
                id: device.id,
                uuid: device.uuid,
                api_key,
            }));
        }
    }

    Ok(None)
}

fn get_device_by_name_endpoint(application_id: u64, name: &str) -> String {
    format!(
        "{}?$filter=belongs_to__application eq '{}' and device_name eq '{}'",
        DEVICE_ENDPOINT, application_id, name
    )
}

pub async fn get_device_by_name(
    token: &str,
    application: &Application,
    name: &str,
) -> Result<Option<Device>> {
    info!("Getting device by name '{}'", name);

    let mut devices = get(token, &get_device_by_name_endpoint(application.id, name))
        .await?
        .json::<Response<Device>>()
        .await?
        .data;

    Ok(devices.pop())
}
