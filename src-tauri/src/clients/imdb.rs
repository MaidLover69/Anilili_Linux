use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ImdbRatingResult {
    pub imdb_id: Option<String>,
    pub rating: Option<String>,
    pub votes: Option<String>,
}

pub struct ImdbClient {
    client: Client,
}

impl ImdbClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_rating(&self, title: &str) -> Option<ImdbRatingResult> {
        let clean_title: String = title
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let query_url = format!(
            "https://v3.sg.media-imdb.com/suggestion/x/{}.json",
            clean_title.trim_matches('_')
        );

        let res = self
            .client
            .get(&query_url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64)")
            .send()
            .await
            .ok()?;

        if !res.status().is_success() {
            return None;
        }

        let json: serde_json::Value = res.json().await.ok()?;
        let suggestions = json["d"].as_array()?;

        for s in suggestions {
            let q_type = s["q"].as_str().unwrap_or_default();
            if q_type == "TV series" || q_type == "feature" || q_type == "TV mini-series" {
                let id = s["id"].as_str().map(String::from);
                return Some(ImdbRatingResult {
                    imdb_id: id,
                    rating: None,
                    votes: None,
                });
            }
        }

        None
    }

    pub async fn fetch_episode_synopses(
        &self,
        title_or_imdb_id: &str,
    ) -> HashMap<i32, String> {
        let mut map = HashMap::new();
        let imdb_id = if title_or_imdb_id.starts_with("tt") {
            title_or_imdb_id.to_string()
        } else {
            match self.fetch_rating(title_or_imdb_id).await {
                Some(r) if r.imdb_id.is_some() => r.imdb_id.unwrap(),
                _ => return map,
            }
        };

        // Fetch Season 1 episodes from IMDb
        let url = format!("https://www.imdb.com/title/{}/episodes/?season=1", imdb_id);
        let res = match self
            .client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            _ => return map,
        };

        let html = match res.text().await {
            Ok(t) => t,
            Err(_) => return map,
        };

        // 1. Try parsing __NEXT_DATA__ JSON
        if let Some(start_idx) = html.find("<script id=\"__NEXT_DATA__\" type=\"application/json\">") {
            let json_start = start_idx + "<script id=\"__NEXT_DATA__\" type=\"application/json\">".len();
            if let Some(end_idx) = html[json_start..].find("</script>") {
                let json_str = &html[json_start..json_start + end_idx];
                if let Ok(next_data) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(items) = next_data
                        .pointer("/props/pageProps/contentData/section/episodes/items")
                        .and_then(|v| v.as_array())
                    {
                        for item in items {
                            let ep_num = item["episode"]
                                .as_str()
                                .and_then(|s| s.parse::<i32>().ok())
                                .or_else(|| item["episode"].as_i64().map(|v| v as i32));
                            let plot = item["plot"].as_str().map(String::from);

                            if let (Some(ep), Some(synopsis)) = (ep_num, plot) {
                                if !synopsis.trim().is_empty() {
                                    map.insert(ep, synopsis);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Regex fallback for episode plot cards
        if map.is_empty() {
            let ep_re = regex::Regex::new(r#"S\d+\.E(\d+)[^<]*</div>[\s\S]*?<div class="ipc-html-content-inner-div"[^>]*>(.*?)</div>"#).unwrap();
            for cap in ep_re.captures_iter(&html) {
                if let Ok(ep_num) = cap[1].parse::<i32>() {
                    let plot = cap[2].trim().to_string();
                    if !plot.is_empty() && !map.contains_key(&ep_num) {
                        map.insert(ep_num, plot);
                    }
                }
            }
        }

        map
    }
}
