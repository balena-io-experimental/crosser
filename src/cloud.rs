use std::io::{stdout, Write};

use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};

use crossterm::cursor::MoveUp;
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};

const API_BASE: &str = "https://api.balena-cloud.com";
const BUILDER_BASE: &str = "https://builder.balena-cloud.com";

const ENDPOINT_APPLICATION: &str = "v5/application";
const BUILD_ENDPOINT: &str = "v3/build";
const REGISTER_ENDPOINT: &str = "device/register";

#[derive(Debug, Deserialize)]
struct Response<T> {
    #[serde(rename = "d")]
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Application {
    id: u64,
    #[serde(rename = "app_name")]
    name: String,
    device_type: String,
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

pub async fn build_application(
    token: &str,
    application: &Application,
    user: &User,
    gzip: Vec<u8>,
) -> Result<bool> {
    let endpoint = get_build_application_endpoint(&user.username, &application.name);
    let url = format!("{}/{}", BUILDER_BASE, endpoint);
    println!("{}", url);
    let mut res = reqwest::Client::new()
        .post(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .header(reqwest::header::CONTENT_ENCODING, "gzip")
        .body(gzip)
        .send()
        .await?;

    let mut stream = ArrayStream::new();

    let mut success = false;

    while let Some(chunk) = res.chunk().await? {
        stream.extend(std::str::from_utf8(&chunk).context("Response is not an utf-8 string")?);
        for value in &mut stream {
            let obj = value
                .as_object()
                .context("Serialized response is not an object")?;

            if let Some(is_success) = obj.get("isSuccess") {
                let is_success = is_success
                    .as_bool()
                    .context("Message isSuccess property is not a boolean")?;
                success = is_success;
            }

            if let Some(message) = obj.get("message") {
                if let Some(replace) = obj.get("replace") {
                    let replace = replace
                        .as_bool()
                        .context("Message replace property is not a boolean")?;
                    if replace {
                        execute!(stdout(), MoveUp(1))?;
                    }
                }
                let message = message
                    .as_str()
                    .context("Response message is not a string")?;
                execute!(stdout(), Print(message), Print('\n'))?;
            }

            if let Some(resource) = obj.get("resource") {
                let resource = resource
                    .as_str()
                    .context("Resource property is not a string")?;

                if resource == "cursor" {
                    let value = obj
                        .get("value")
                        .context("No replace property defined")?
                        .as_str()
                        .context("Value is not a string")?;
                    if value == "erase" {
                        execute!(stdout(), MoveUp(1), Clear(ClearType::CurrentLine))?;
                    }
                }
            }
        }
    }

    Ok(success)
}

pub struct ArrayStream {
    buffer: String,
    started: bool,
}

impl ArrayStream {
    pub fn new() -> Self {
        ArrayStream {
            buffer: String::new(),
            started: false,
        }
    }

    pub fn extend(&mut self, other: &str) {
        self.buffer.push_str(other);
    }

    fn next_value(&mut self) -> (Option<Value>, usize) {
        let mut cut: usize = 0;

        for (i, ch) in self.buffer.chars().enumerate() {
            match ch {
                '\n' => continue,
                '[' if !self.started => {
                    self.started = true;
                    continue;
                }
                ',' if self.started => continue,
                _ => {
                    cut = i;
                    break;
                }
            }
        }

        let substring = &self.buffer[cut..];

        let mut stream = Deserializer::from_str(substring).into_iter::<Value>();

        match stream.next() {
            Some(result) => (result.ok(), cut + stream.byte_offset()),
            None => (None, cut),
        }
    }
}

impl Iterator for ArrayStream {
    type Item = Value;

    fn next(&mut self) -> Option<Value> {
        let (value_option, byte_offset) = self.next_value();
        self.buffer.drain(..byte_offset);
        value_option
    }
}

fn get_build_application_endpoint(username: &str, app: &str) -> String {
    format!(
        "{}?owner={}&app={}&dockerfilePath=&emulated=false&nocache=false&headless=false",
        BUILD_ENDPOINT, username, app
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
