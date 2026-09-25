use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static DOH_CACHE: Lazy<Mutex<HashMap<String, (IpAddr, Instant)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn resolve_doh(domain: &str, provider: &str) -> Option<IpAddr> {
    // Check cache
    if let Ok(guard) = DOH_CACHE.lock() {
        if let Some((ip, expiry)) = guard.get(domain) {
            if Instant::now() < *expiry {
                return Some(*ip);
            }
        }
    }

    let doh_url = match provider.to_lowercase().as_str() {
        "google" => format!("https://dns.google/resolve?name={}&type=A", domain),
        "adguard" => format!("https://dns.adguard-dns.com/resolve?name={}&type=A", domain),
        _ => format!("https://cloudflare-dns.com/dns-query?name={}&type=A", domain),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .ok()?;

    let res = client
        .get(&doh_url)
        .header("Accept", "application/dns-json")
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let json: Value = res.json().await.ok()?;
    if let Some(answers) = json.get("Answer").and_then(|a| a.as_array()) {
        for ans in answers {
            if let Some(data) = ans.get("data").and_then(|d| d.as_str()) {
                if let Ok(ip) = data.parse::<IpAddr>() {
                    // Cache for 5 minutes
                    if let Ok(mut guard) = DOH_CACHE.lock() {
                        guard.insert(
                            domain.to_string(),
                            (ip, Instant::now() + Duration::from_secs(300)),
                        );
                    }
                    return Some(ip);
                }
            }
        }
    }

    None
}

pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(10))
        .gzip(true)
        .brotli(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
