use anyhow::Result;

use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.balena-cloud.com";

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    #[serde(rename = "d")]
    pub data: Vec<T>,
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

pub async fn patch<T: Serialize + ?Sized>(
    token: &str,
    endpoint: &str,
    json: &T,
) -> Result<reqwest::Response> {
    let url = format!("{}/{}", API_BASE, endpoint);
    Ok(reqwest::Client::new()
        .patch(&url)
        .json(json)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?)
}
