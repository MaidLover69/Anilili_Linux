use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub struct KickAssAnimeProvider;

impl KickAssAnimeProvider {
    pub fn new() -> Self {
        Self
    }

    fn build_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            .build()
            .unwrap_or_else(|_| Client::new())
    }
}

#[async_trait]
impl AnimeProvider for KickAssAnimeProvider {
    fn name(&self) -> &'static str {
        "kaa"
    }

    fn supports_dub(&self) -> bool {
        true
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let client = Self::build_client();
        let query = title_romaji.unwrap_or_else(|| "").trim();
        let query_str = if query.is_empty() {
            anilist_id.to_string()
        } else {
            query.to_string()
        };

        let mirrors = ["https://kaa.lt", "https://kaa1.lt", "https://kaa2.lt"];

        for mirror in &mirrors {
            // Step 1: POST /api/fsearch with title query
            let search_url = format!("{}/api/fsearch", mirror);
            let search_body = serde_json::json!({ "query": query_str });
            let search_res = client
                .post(&search_url)
                .header("Referer", format!("{}/", mirror))
                .header("Origin", *mirror)
                .header("Content-Type", "application/json")
                .json(&search_body)
                .send()
                .await;

            let show_slug = match search_res {
                Ok(r) if r.status().is_success() => {
                    if let Ok(json) = r.json::<serde_json::Value>().await {
                        if let Some(arr) = json["result"].as_array().or_else(|| json.as_array()) {
                            arr.first().and_then(|item| item["slug"].as_str().map(String::from))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let show_slug = match show_slug {
                Some(s) => s,
                None => continue,
            };

            // Step 2: Fetch all episode pages from /api/show/{slug}/episodes
            let mut sub_eps: Vec<EpisodeItem> = Vec::new();
            let mut dub_eps: Vec<EpisodeItem> = Vec::new();

            for (lang, target_vec) in [("ja-JP", &mut sub_eps), ("en-US", &mut dub_eps)] {
                let mut page = 1;
                loop {
                    let ep_url = format!("{}/api/show/{}/episodes?page={}&lang={}", mirror, show_slug, page, lang);
                    let ep_res = client
                        .get(&ep_url)
                        .header("Referer", format!("{}/", mirror))
                        .send()
                        .await;

                    let json: serde_json::Value = match ep_res {
                        Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
                        _ => break,
                    };

                    let result_obj = &json["result"];
                    let ep_array = if let Some(arr) = result_obj.as_array() {
                        arr
                    } else if let Some(arr) = result_obj["episodes"].as_array() {
                        arr
                    } else if let Some(arr) = json["episodes"].as_array() {
                        arr
                    } else {
                        break;
                    };

                    if ep_array.is_empty() {
                        break;
                    }

                    for item in ep_array {
                        let ep_num = item["episode_number"]
                            .as_f64()
                            .or_else(|| item["number"].as_f64())
                            .unwrap_or(target_vec.len() as f64 + 1.0);

                        let ep_slug = item["slug"]
                            .as_str()
                            .map(String::from)
                            .unwrap_or_else(|| format!("ep-{}", ep_num as i32));

                        let title = item["title"]
                            .as_str()
                            .map(String::from)
                            .or_else(|| Some(format!("Episode {}", ep_num as i32)));

                        if !target_vec.iter().any(|e| (e.number - ep_num).abs() < 0.01) {
                            target_vec.push(EpisodeItem {
                                pipe_id: format!("{}|{}|{}", mirror, show_slug, ep_slug),
                                number: ep_num,
                                title,
                                image: None,
                                synopsis: None,
                                filler: false,
                            });
                        }
                    }

                    let has_next = result_obj["hasNextPage"].as_bool()
                        .or_else(|| json["hasNextPage"].as_bool())
                        .unwrap_or(false);

                    let total_pages = result_obj["pages"].as_array().map(|a| a.len()).unwrap_or(0);
                    if !has_next && (total_pages == 0 || page >= total_pages) {
                        break;
                    }
                    page += 1;
                    if page > 100 {
                        break;
                    }
                }
            }

            sub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));
            dub_eps.sort_by(|a, b| a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal));

            if !sub_eps.is_empty() || !dub_eps.is_empty() {
                if dub_eps.is_empty() {
                    dub_eps = sub_eps.clone();
                }
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: sub_eps,
                    dub: dub_eps,
                });
            }
        }

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: Vec::new(),
            dub: Vec::new(),
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        // pipe_id format: "{mirror}|{show_slug}|{ep_slug}"
        let parts: Vec<&str> = episode.pipe_id.splitn(3, '|').collect();
        if parts.len() < 3 {
            return Err(AppError::NoSourcesFound);
        }
        let (mirror, show_slug, ep_slug) = (parts[0], parts[1], parts[2]);

        let client = Self::build_client();

        // Episode detail endpoint: /api/show/{show_slug}/episode/{ep_slug}
        let detail_url = if ep_slug.starts_with("ep-") {
            format!("{}/api/show/{}/episode/{}", mirror, show_slug, ep_slug)
        } else {
            format!("{}/api/show/{}/episode/ep-{}-{}", mirror, show_slug, episode.number as i32, ep_slug)
        };

        let res = client
            .get(&detail_url)
            .header("Referer", format!("{}/", mirror))
            .send()
            .await;

        if let Ok(r) = res {
            if r.status().is_success() {
                if let Ok(json) = r.json::<serde_json::Value>().await {
                    let text = json.to_string();

                    // Look for player iframe or player embed URL (e.g. CatStream / krussdomi)
                    let player_re = regex::Regex::new(r#"https?://[^\s"'<>]+?(?:krussdomi|cat-player|player)[^\s"'<>]*"#).unwrap();
                    if let Some(player_match) = player_re.find(&text) {
                        let player_url = player_match.as_str().replace("\\/", "/");
                        let player_res = client
                            .get(&player_url)
                            .header("Referer", format!("{}/", mirror))
                            .header("Origin", mirror)
                            .send()
                            .await;

                        if let Ok(pr) = player_res {
                            if let Ok(player_html) = pr.text().await {
                                let m3u8_re = regex::Regex::new(r#"https?://[^\s"'<>]+?\.m3u8[^\s"'<>]*"#).unwrap();
                                if let Some(mat) = m3u8_re.find(&player_html) {
                                    let m3u8_url = mat.as_str().replace("\\/", "/");
                                    return Ok(SourcesResult {
                                        streams: vec![StreamItem {
                    subtitle_variant: None,
                                            url: m3u8_url,
                                            stream_type: "hls".to_string(),
                                            quality: Some("auto".to_string()),
                                            audio: None,
                                            referer: Some("https://krussdomi.com/".to_string()),
                                            is_active: true,
                                            origin: None,
                                            headers: None,
                                        }],
                                        subtitles: vec![],
                                        skip: None,
                                    });
                                }
                            }
                        }
                    }

                    // Direct m3u8 extraction fallback from detail response
                    let m3u8_re = regex::Regex::new(r#"https?://[^\s"'<>]+?\.m3u8[^\s"'<>]*"#).unwrap();
                    if let Some(mat) = m3u8_re.find(&text) {
                        let m3u8_url = mat.as_str().replace("\\/", "/");
                        return Ok(SourcesResult {
                            streams: vec![StreamItem {
                    subtitle_variant: None,
                                url: m3u8_url,
                                stream_type: "hls".to_string(),
                                quality: Some("auto".to_string()),
                                audio: None,
                                referer: Some(format!("{}/", mirror)),
                                is_active: true,
                                origin: None,
                                headers: None,
                            }],
                            subtitles: vec![],
                            skip: None,
                        });
                    }
                }
            }
        }

        Err(AppError::NoSourcesFound)
    }
}

