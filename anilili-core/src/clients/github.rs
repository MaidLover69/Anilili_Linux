use pyo3::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateInfo {
    #[pyo3(get, set)]
    pub version: String,
    #[pyo3(get, set)]
    pub changelog: String,
    #[pyo3(get, set)]
    pub published_at: String,
    #[pyo3(get, set)]
    pub release_url: String,
}

#[pymethods]
impl UpdateInfo {
    #[new]
    #[pyo3(signature = (version, changelog, published_at, release_url))]
    pub fn new(version: String, changelog: String, published_at: String, release_url: String) -> Self {
        Self {
            version,
            changelog,
            published_at,
            release_url,
        }
    }
}

pub async fn fetch_latest_release(client: &Client) -> Option<UpdateInfo> {
    let url = "https://api.github.com/repos/kompoti121/Anilili/releases/latest";
    let res = client
        .get(url)
        .header("User-Agent", "AnililiLinux/1.0")
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let json: serde_json::Value = res.json().await.ok()?;
    let raw_tag = json["tag_name"].as_str().unwrap_or("v1.0.0");
    if raw_tag.to_uppercase().contains("APK") {
        return None;
    }
    let version = raw_tag.trim_start_matches('v').to_string();
    let changelog = json["body"].as_str().unwrap_or("No changelog available.").to_string();
    let published_at = json["published_at"].as_str().unwrap_or("").to_string();
    let release_url = json["html_url"]
        .as_str()
        .unwrap_or("https://github.com/kompoti121/Anilili/releases")
        .to_string();

    Some(UpdateInfo {
        version,
        changelog,
        published_at,
        release_url,
    })
}

