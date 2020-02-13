use std::io::{stdout, Write};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Deserializer, Value};

use crossterm::cursor::MoveUp;
use crossterm::execute;
use crossterm::style::Print;

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

async fn get(token: &str, endpoint: &str) -> Result<reqwest::Response> {
    let url = format!("{}/{}", API_BASE, endpoint);
    Ok(reqwest::Client::new()
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?)
}

pub async fn get_application_by_name(token: &str, app: &str) -> Result<Vec<Application>> {
    Ok(get(token, &get_application_by_name_endpoint(app))
        .await?
        .json::<Response<Application>>()
        .await?
        .data)
}

pub async fn get_application_username(token: &str, app: &str) -> Result<String> {
    let mut users = get(token, &get_application_username_endpoint(app))
        .await?
        .json::<Response<Value>>()
        .await?
        .data;

    Ok(users
        .pop()
        .context("One application owner expected")?
        .pointer("/user/0/username")
        .context("Username pointer failed")?
        .as_str()
        .context("Usename not a string")?
        .to_string())
}

fn get_application_username_endpoint(app: &str) -> String {
    format!(
        "{}?$expand=user($select=username)&$filter=app_name eq '{}'&$select=id",
        ENDPOINT_APPLICATION, app
    )
}

pub async fn build_application(
    token: &str,
    username: &str,
    app: &str,
    gzip: Vec<u8>,
) -> Result<bool> {
    let endpoint = get_build_application_endpoint(username, app);
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
