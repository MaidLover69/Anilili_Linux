use crate::clients::http::build_http_client;
use crate::error::AppError;
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkipTimes {
    pub intro_start: Option<f64>,
    pub intro_end: Option<f64>,
    pub outro_start: Option<f64>,
    pub outro_end: Option<f64>,
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

#[pyfunction]
#[pyo3(signature = (mal_id, episode_number, duration_s, callback=None))]
pub fn get_skip_times(
    mal_id: i64,
    episode_number: f64,
    duration_s: f64,
    callback: Option<PyObject>,
) -> PyResult<()> {
    if let Some(cb) = callback {
        RUNTIME.spawn(async move {
            let res = fetch_skip_times(mal_id, episode_number, duration_s).await;
            Python::with_gil(|py| match res {
                Ok(times) => {
                    let dict = PyDict::new(py);
                    let _ = dict.set_item("intro_start", times.intro_start);
                    let _ = dict.set_item("intro_end", times.intro_end);
                    let _ = dict.set_item("outro_start", times.outro_start);
                    let _ = dict.set_item("outro_end", times.outro_end);
                    let none_err: Option<String> = None;
                    let _ = cb.call1(py, (true, dict, none_err));
                }
                Err(err) => {
                    let err_str = Some(err.to_string());
                    let empty_dict = PyDict::new(py);
                    let _ = cb.call1(py, (false, empty_dict, err_str));
                }
            });
        });
    }
    Ok(())
}
