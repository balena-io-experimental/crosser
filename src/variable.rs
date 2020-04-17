use anyhow::Result;
use log::info;

use serde::{Deserialize, Serialize};

use crate::api::{get, post, Response};

const ENDPOINT_DEVICE_VARIABLES: &str = "v5/device_environment_variable";

const API_KEY: &str = "API_KEY";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DeviceEnvironmentVariableData {
    device: String,
    name: String,
    value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Variable {
    id: u64,
    name: String,
    value: String,
}

fn get_device_environment_variable_endpoint(device_id: u64, name: &str) -> String {
    format!(
        "{}?$filter=device eq '{}' and name eq '{}'",
        ENDPOINT_DEVICE_VARIABLES, device_id, name
    )
}

async fn get_device_environment_variable(
    token: &str,
    device_id: u64,
    name: &str,
) -> Result<Option<String>> {
    info!("Getting device environment variable '{}'", name);

    let mut variables = get(
        token,
        &get_device_environment_variable_endpoint(device_id, name),
    )
    .await?
    .json::<Response<Variable>>()
    .await?
    .data;

    if let Some(variable) = variables.pop() {
        Ok(Some(variable.value))
    } else {
        Ok(None)
    }
}

async fn store_device_environment_variable(
    token: &str,
    device_id: u64,
    name: &str,
    value: &str,
) -> Result<()> {
    let variable_data = DeviceEnvironmentVariableData {
        device: format!("{}", device_id),
        name: name.to_string(),
        value: value.to_string(),
    };

    let _result = post(token, ENDPOINT_DEVICE_VARIABLES, &variable_data).await?;

    info!("Stored `{}` device variable", name);

    Ok(())
}

pub async fn get_device_api_key(token: &str, device_id: u64) -> Result<Option<String>> {
    get_device_environment_variable(token, device_id, API_KEY).await
}

pub async fn store_device_api_key(token: &str, device_id: u64, value: &str) -> Result<()> {
    store_device_environment_variable(token, device_id, API_KEY, value).await
}
