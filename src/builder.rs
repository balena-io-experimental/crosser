use anyhow::{Context, Result};

use std::io::{stdout, Write};

use crossterm::cursor::MoveUp;
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};

use serde_json::{Deserializer, Value};

use crate::application::Application;
use crate::application::User;

const BUILDER_BASE: &str = "https://builder.balena-cloud.com";
const BUILD_ENDPOINT: &str = "v3/build";

pub async fn build_application(
    token: &str,
    application: &Application,
    user: &User,
    gzip: Vec<u8>,
) -> Result<bool> {
    let endpoint = get_build_application_endpoint(&user.username, &application.name);
    let url = format!("{}/{}", BUILDER_BASE, endpoint);
    println!("{}", url);
    let response = reqwest::Client::new()
        .post(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .header(reqwest::header::CONTENT_ENCODING, "gzip")
        .body(gzip)
        .send()
        .await?;

    parse_build_stream(response).await
}

async fn parse_build_stream(mut response: reqwest::Response) -> Result<bool> {
    let mut stream = ArrayStream::new();

    let mut success = false;

    while let Some(chunk) = response.chunk().await? {
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
