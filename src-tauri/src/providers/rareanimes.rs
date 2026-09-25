use crate::error::AppError;
use crate::models::{Category, EpisodeItem, ProviderData, SourcesResult, StreamItem};
use crate::providers::AnimeProvider;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Client;
use std::time::Duration;

pub struct RareAnimesProvider;

impl RareAnimesProvider {
    pub fn new() -> Self {
        Self
    }

    fn build_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0")
            // Follow redirects (needed for the codedew.com zipper chain)
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap_or_else(|_| Client::new())
    }

    /// Decode `_juicycodes` field: triple base64 decode → UTF-8 string → extract HLS URL
    fn decode_juicycodes(encoded: &str) -> Option<String> {
        // RareAnimes encodes the HLS config with 3 rounds of base64
        let mut data = encoded.to_string();
        for _ in 0..3 {
            data = String::from_utf8(BASE64.decode(data.trim()).ok()?).ok()?;
        }
        // After decoding we get a JW Player sources config JSON string
        // Extract the .m3u8 URL from it
        let m3u8_re = regex::Regex::new(r#"https?://[^\s"'\\<>]+?\.m3u8[^\s"'\\<>]*"#).ok()?;
        m3u8_re.find(&data).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl AnimeProvider for RareAnimesProvider {
    fn name(&self) -> &'static str {
        "rareanimes"
    }

    fn supports_dub(&self) -> bool {
        // RareAnimes specialises in Hindi/Tamil/Telugu dub
        true
    }

    async fn get_episodes(
        &self,
        anilist_id: i64,
        _mal_id: Option<i64>,
        _title_romaji: Option<&str>,
    ) -> Result<ProviderData, AppError> {
        let client = Self::build_client();

        // Step 1: WordPress search by AniList ID (used as search keyword)
        let search_url = format!("https://rareanim.es/?s={}", anilist_id);
        let search_res = client
            .get(&search_url)
            .header("Referer", "https://rareanim.es/")
            .send()
            .await;

        let search_html = match search_res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: Vec::new(),
                    dub: Vec::new(),
                })
            }
        };

        // Extract first result slug from search results page
        // Pattern: href="https://rareanim.es/anime/{slug}/"
        let slug_re = match regex::Regex::new(
            r#"href="(https://rareanim\.es/anime/([^/"]+)/?)"#,
        ) {
            Ok(r) => r,
            Err(_) => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: Vec::new(),
                    dub: Vec::new(),
                })
            }
        };

        let anime_url = match slug_re.captures(&search_html) {
            Some(cap) => cap[1].to_string(),
            None => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: Vec::new(),
                    dub: Vec::new(),
                })
            }
        };

        // Step 2: Fetch the anime page to get episode list
        let anime_res = client
            .get(&anime_url)
            .header("Referer", "https://rareanim.es/")
            .send()
            .await;

        let anime_html = match anime_res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: Vec::new(),
                    dub: Vec::new(),
                })
            }
        };

        // Parse episode links: href="https://rareanim.es/{episode-slug}/"
        let ep_re = match regex::Regex::new(
            r#"href="(https://rareanim\.es/[^"]+?episode[^"]+?)"#,
        ) {
            Ok(r) => r,
            Err(_) => {
                return Ok(ProviderData {
                    name: self.name().to_string(),
                    sub: Vec::new(),
                    dub: Vec::new(),
                })
            }
        };

        // Also extract episode number from the URL slug
        let num_re = regex::Regex::new(r"episode[- ]?(\d+(?:-\d+)?)").unwrap_or_else(|_| {
            regex::Regex::new(r"(\d+)").unwrap()
        });

        let mut sub_eps: Vec<EpisodeItem> = Vec::new();
        for cap in ep_re.captures_iter(&anime_html) {
            let ep_url = cap[1].to_string();
            let ep_num = num_re
                .captures(&ep_url)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(sub_eps.len() as f64 + 1.0);

            if !sub_eps.iter().any(|e| (e.number - ep_num).abs() < 0.01) {
                sub_eps.push(EpisodeItem {
                    pipe_id: format!("rareanimes|{}", ep_url),
                    number: ep_num,
                    title: Some(format!("Episode {}", ep_num as i32)),
                    image: None,
                    synopsis: None,
                    filler: false,
                });
            }
        }

        sub_eps.sort_by(|a, b| {
            a.number
                .partial_cmp(&b.number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(ProviderData {
            name: self.name().to_string(),
            sub: sub_eps.clone(),
            // RareAnimes typically has dubbed episodes in the same list
            dub: sub_eps,
        })
    }

    async fn get_sources(
        &self,
        episode: &EpisodeItem,
        _category: Category,
    ) -> Result<SourcesResult, AppError> {
        // pipe_id format: "rareanimes|{ep_url}"
        let parts: Vec<&str> = episode.pipe_id.splitn(2, '|').collect();
        if parts.len() < 2 || parts[0] != "rareanimes" {
            return Err(AppError::NoSourcesFound);
        }
        let ep_url = parts[1];

        let client = Self::build_client();

        // Step 1: Fetch the episode page
        let ep_res = client
            .get(ep_url)
            .header("Referer", "https://rareanim.es/")
            .send()
            .await;

        let ep_html = match ep_res {
            Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
            _ => return Err(AppError::NoSourcesFound),
        };

        // Step 2: Extract the codedew.com zipper URL
        // Pattern: href="https://codedew.com/zipper/?url=..."
        let zipper_re = match regex::Regex::new(
            r#"href="(https://codedew\.com/zipper/\?url=[^"]+)""#,
        ) {
            Ok(r) => r,
            Err(_) => return Err(AppError::NoSourcesFound),
        };

        if let Some(cap) = zipper_re.captures(&ep_html) {
            let zipper_url = cap[1].to_string();

            // Step 3: Follow the zipper redirect chain
            let zipper_res = client
                .get(&zipper_url)
                .header("Referer", ep_url)
                .send()
                .await;

            let player_html = match zipper_res {
                Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
                _ => return Err(AppError::NoSourcesFound),
            };

            // Step 4: Extract _juicycodes encoded field from JW Player config
            let juice_re = match regex::Regex::new(r#"_juicycodes\s*[=:]\s*['"]([\w+/=]+)['"]"#) {
                Ok(r) => r,
                Err(_) => return Err(AppError::NoSourcesFound),
            };

            if let Some(cap) = juice_re.captures(&player_html) {
                let encoded = cap[1].to_string();
                if let Some(hls_url) = Self::decode_juicycodes(&encoded) {
                    return Ok(SourcesResult {
                        streams: vec![StreamItem {
                    subtitle_variant: None,
                            url: hls_url,
                            stream_type: "hls".to_string(),
                            quality: Some("auto".to_string()),
                            audio: None,
                            referer: Some("https://codedew.com/".to_string()),
                            is_active: true,
                            origin: None,
                            headers: None,
                        }],
                        subtitles: vec![],
                        skip: None,
                    });
                }
            }

            // Fallback: regex extract any .m3u8 directly from player HTML
            let m3u8_re = match regex::Regex::new(
                r#"https?://[^\s"'\\<>]+?\.m3u8[^\s"'\\<>]*"#,
            ) {
                Ok(r) => r,
                Err(_) => return Err(AppError::NoSourcesFound),
            };
            if let Some(mat) = m3u8_re.find(&player_html) {
                return Ok(SourcesResult {
                    streams: vec![StreamItem {
                    subtitle_variant: None,
                        url: mat.as_str().to_string(),
                        stream_type: "hls".to_string(),
                        quality: Some("auto".to_string()),
                        audio: None,
                        referer: Some("https://codedew.com/".to_string()),
                        is_active: true,
                        origin: None,
                        headers: None,
                    }],
                    subtitles: vec![],
                    skip: None,
                });
            }
        }

        // Last-resort: look for .m3u8 directly on the episode page itself
        let m3u8_re = match regex::Regex::new(
            r#"https?://[^\s"'\\<>]+?\.m3u8[^\s"'\\<>]*"#,
        ) {
            Ok(r) => r,
            Err(_) => return Err(AppError::NoSourcesFound),
        };
        if let Some(mat) = m3u8_re.find(&ep_html) {
            return Ok(SourcesResult {
                streams: vec![StreamItem {
                    subtitle_variant: None,
                    url: mat.as_str().to_string(),
                    stream_type: "hls".to_string(),
                    quality: Some("auto".to_string()),
                    audio: None,
                    referer: Some("https://rareanim.es/".to_string()),
                    is_active: true,
                    origin: None,
                    headers: None,
                }],
                subtitles: vec![],
                skip: None,
            });
        }

        Err(AppError::NoSourcesFound)
    }
}
