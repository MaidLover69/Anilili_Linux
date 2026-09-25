use crate::clients::http::build_http_client;
use crate::error::AppError;
use crate::models::SkipTimes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipInterval {
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipResult {
    #[serde(rename = "skipType")]
    pub skip_type: String,
    pub interval: SkipInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniSkipResponse {
    pub found: bool,
    pub results: Option<Vec<SkipResult>>,
}

pub async fn fetch_skip_times(
    mal_id: i64,
    episode_number: f64,
    duration_s: f64,
) -> Result<SkipTimes, AppError> {
    if duration_s <= 60.0 {
        return Ok(SkipTimes::default());
    }

    let client = build_http_client();
    let url = format!(
        "https://api.aniskip.com/v2/skip-times/{}/{}?types[]=op&types[]=ed&episodeLength={}",
        mal_id, episode_number as i32, duration_s as i32
    );

    let resp = client.get(&url).send().await;
    let response = match resp {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(SkipTimes::default()),
    };

    let data: AniSkipResponse = match response.json().await {
        Ok(d) => d,
        Err(_) => return Ok(SkipTimes::default()),
    };

    let mut times = SkipTimes::default();
    if let Some(results) = data.results {
        for res in results {
            if res.skip_type == "op" {
                times.intro_start = Some(res.interval.start_time);
                times.intro_end = Some(res.interval.end_time);
            } else if res.skip_type == "ed" {
                times.outro_start = Some(res.interval.start_time);
                times.outro_end = Some(res.interval.end_time);
            }
        }
    }

    Ok(times)
}
