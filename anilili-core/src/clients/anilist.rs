use crate::error::AppError;
use crate::models::{AiringEntry, CoverImage, Media, MediaTitle};
use reqwest::Client;
use serde_json::json;

pub const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co";


const MEDIA_FRAGMENT: &str = r#"
id
idMal
title {
    romaji
    english
    native
    userPreferred
}
coverImage {
    large
    extraLarge
    color
}
bannerImage
description
format
season
seasonYear
episodes
duration
status
averageScore
meanScore
popularity
favourites
isAdult
genres
"#;

const MEDIA_DETAILS_QUERY: &str = r#"
query ($id: Int) {
    Media(id: $id, type: ANIME) {
        id
        idMal
        title {
            romaji
            english
            native
            userPreferred
        }
        coverImage {
            large
            extraLarge
            color
        }
        bannerImage
        description
        format
        season
        seasonYear
        episodes
        duration
        status
        averageScore
        meanScore
        popularity
        favourites
        isAdult
        genres
        nextAiringEpisode {
            id
            airingAt
            timeUntilAiring
            episode
        }
        studios {
            edges {
                isMain
                node {
                    id
                    name
                }
            }
        }
        relations {
            edges {
                relationType
                node {
                    id
                    idMal
                    title {
                        romaji
                        english
                        native
                        userPreferred
                    }
                    coverImage {
                        large
                        extraLarge
                        color
                    }
                    format
                    status
                    averageScore
                }
            }
        }
    }
}
"#;

pub struct AniListClient {
    client: Client,
}

impl AniListClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_media_list(
        &self,
        sort: &str,
        per_page: i32,
    ) -> Result<Vec<Media>, AppError> {
        let query = format!(
            r#"
            query ($sort: [MediaSort], $perPage: Int) {{
                Page(perPage: $perPage) {{
                    media(sort: $sort, type: ANIME) {{
                        {}
                    }}
                }}
            }}
            "#,
            MEDIA_FRAGMENT
        );

        let body = json!({
            "query": query,
            "variables": {
                "sort": [sort],
                "perPage": per_page
            }
        });

        let res = self
            .client
            .post(ANILIST_GRAPHQL_URL)
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(AppError::Parse(format!(
                "AniList API returned error status {}: {}",
                status, text
            )));
        }

        let json_val: serde_json::Value = res.json().await?;
        let media_arr = json_val["data"]["Page"]["media"]
            .as_array()
            .ok_or_else(|| AppError::Parse("Invalid media array response".to_string()))?;

        let mut list = Vec::new();
        for item in media_arr {
            if let Ok(media) = parse_media_json(item) {
                list.push(media);
            }
        }
        Ok(list)
    }

    pub async fn search(
        &self,
        query: Option<String>,
        genres: Vec<String>,
        format: Option<String>,
        status: Option<String>,
        sort: Option<String>,
        page: i32,
        per_page: i32,
    ) -> Result<Vec<Media>, AppError> {
        let gql_query = format!(
            r#"
            query ($page: Int, $perPage: Int, $search: String, $genre_in: [String], $format: MediaFormat, $status: MediaStatus, $sort: [MediaSort]) {{
                Page(page: $page, perPage: $perPage) {{
                    media(search: $search, genre_in: $genre_in, format: $format, status: $status, sort: $sort, type: ANIME) {{
                        {}
                    }}
                }}
            }}
            "#,
            MEDIA_FRAGMENT
        );

        let mut variables = serde_json::Map::new();
        variables.insert("page".to_string(), json!(page));
        variables.insert("perPage".to_string(), json!(per_page));

        if let Some(q) = query {
            if !q.trim().is_empty() {
                variables.insert("search".to_string(), json!(q));
            }
        }
        if !genres.is_empty() {
            variables.insert("genre_in".to_string(), json!(genres));
        }
        if let Some(f) = format {
            if !f.trim().is_empty() {
                variables.insert("format".to_string(), json!(f));
            }
        }
        if let Some(s) = status {
            if !s.trim().is_empty() {
                variables.insert("status".to_string(), json!(s));
            }
        }
        let sort_val = sort.unwrap_or_else(|| "POPULARITY_DESC".to_string());
        variables.insert("sort".to_string(), json!([sort_val]));

        let body = json!({
            "query": gql_query,
            "variables": variables
        });

        let res = self
            .client
            .post(ANILIST_GRAPHQL_URL)
            .json(&body)
            .send()
            .await?;

        let json_val: serde_json::Value = res.json().await?;
        let media_arr = json_val["data"]["Page"]["media"]
            .as_array()
            .ok_or_else(|| AppError::Parse("Invalid search media array response".to_string()))?;

        let mut list = Vec::new();
        for item in media_arr {
            if let Ok(media) = parse_media_json(item) {
                list.push(media);
            }
        }
        Ok(list)
    }

    pub async fn fetch_details(&self, id: i64) -> Result<serde_json::Value, AppError> {
        let body = json!({
            "query": MEDIA_DETAILS_QUERY,
            "variables": {
                "id": id
            }
        });

        let res = self
            .client
            .post(ANILIST_GRAPHQL_URL)
            .json(&body)
            .send()
            .await?;

        let json_val: serde_json::Value = res.json().await?;
        let media_obj = &json_val["data"]["Media"];
        if media_obj.is_null() {
            return Err(AppError::Parse(format!("Anime with ID {} not found", id)));
        }
        Ok(media_obj.clone())
    }

    pub async fn get_viewer(&self, access_token: &str) -> Result<crate::models::Viewer, AppError> {
        let query = r#"
            query {
                Viewer {
                    id
                    name
                    avatar {
                        large
                    }
                    statistics {
                        anime {
                            count
                            episodesWatched
                            minutesWatched
                            meanScore
                        }
                    }
                }
            }
        "#;

        let body = json!({ "query": query });
        let res = self
            .client
            .post(ANILIST_GRAPHQL_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&body)
            .send()
            .await?;

        let json_val: serde_json::Value = res.json().await?;
        let viewer_val = &json_val["data"]["Viewer"];
        if viewer_val.is_null() {
            return Err(AppError::Parse("Viewer query returned null".to_string()));
        }

        let id = viewer_val["id"].as_i64().unwrap_or(0) as i32;
        let name = viewer_val["name"].as_str().unwrap_or("").to_string();
        let avatar_url = viewer_val["avatar"]["large"].as_str().map(String::from);
        let anime_stats = &viewer_val["statistics"]["anime"];
        let anime_count = anime_stats["count"].as_i64().unwrap_or(0) as i32;
        let episodes_watched = anime_stats["episodesWatched"].as_i64().unwrap_or(0) as i32;
        let minutes_watched = anime_stats["minutesWatched"].as_i64().unwrap_or(0);
        let mean_score = anime_stats["meanScore"].as_f64().unwrap_or(0.0);

        Ok(crate::models::Viewer {
            id,
            name,
            avatar_url,
            anime_count,
            episodes_watched,
            minutes_watched,
            mean_score,
        })
    }

    pub async fn get_user_library(
        &self,
        access_token: &str,
        user_id: i32,
        status_filter: Option<String>,
    ) -> Result<Vec<crate::models::MediaListEntry>, AppError> {
        let query = r#"
            query ($userId: Int, $status: MediaListStatus) {
                MediaListCollection(userId: $userId, type: ANIME, status: $status) {
                    lists {
                        name
                        status
                        entries {
                            id
                            mediaId
                            status
                            progress
                            score
                            media {
                                id
                                title {
                                    userPreferred
                                    english
                                    romaji
                                }
                                coverImage {
                                    large
                                    extraLarge
                                }
                                format
                                episodes
                                averageScore
                            }
                        }
                    }
                }
            }
        "#;

        let mut variables = serde_json::Map::new();
        variables.insert("userId".to_string(), json!(user_id));
        if let Some(ref s) = status_filter {
            if !s.trim().is_empty() {
                variables.insert("status".to_string(), json!(s));
            }
        }

        let body = json!({
            "query": query,
            "variables": variables
        });

        let res = self
            .client
            .post(ANILIST_GRAPHQL_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&body)
            .send()
            .await?;

        let json_val: serde_json::Value = res.json().await?;
        let lists = json_val["data"]["MediaListCollection"]["lists"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut result = Vec::new();
        for list in lists {
            if let Some(entries) = list["entries"].as_array() {
                for item in entries {
                    let entry_id = item["id"].as_i64().unwrap_or(0) as i32;
                    let media_id = item["mediaId"].as_i64().unwrap_or(0) as i32;
                    let progress = item["progress"].as_i64().unwrap_or(0) as i32;
                    let score = item["score"].as_f64().unwrap_or(0.0);
                    let status_str = item["status"].as_str().map(String::from);

                    let media_val = &item["media"];
                    let title = media_val["title"]["userPreferred"]
                        .as_str()
                        .or_else(|| media_val["title"]["english"].as_str())
                        .or_else(|| media_val["title"]["romaji"].as_str())
                        .map(String::from);
                    let cover = media_val["coverImage"]["extraLarge"]
                        .as_str()
                        .or_else(|| media_val["coverImage"]["large"].as_str())
                        .map(String::from);
                    let format_str = media_val["format"].as_str().map(String::from);
                    let total_episodes = media_val["episodes"].as_i64().map(|v| v as i32);
                    let average_score = media_val["averageScore"].as_i64().map(|v| v as i32);

                    result.push(crate::models::MediaListEntry {
                        id: if media_id > 0 { media_id } else { entry_id },
                        progress,
                        score,
                        status: status_str,
                        title,
                        cover,
                        format_str,
                        total_episodes,
                        average_score,
                    });
                }
            }
        }

        Ok(result)
    }

    pub async fn save_list_entry(
        &self,
        access_token: &str,
        media_id: i32,
        status: Option<String>,
        progress: Option<i32>,
        score: Option<f64>,
    ) -> Result<crate::models::MediaListEntry, AppError> {
        let mutation = r#"
            mutation ($mediaId: Int, $status: MediaListStatus, $progress: Int, $score: Float) {
                SaveMediaListEntry(mediaId: $mediaId, status: $status, progress: $progress, score: $score) {
                    id
                    mediaId
                    status
                    progress
                    score
                }
            }
        "#;

        let mut variables = serde_json::Map::new();
        variables.insert("mediaId".to_string(), json!(media_id));
        if let Some(ref s) = status {
            variables.insert("status".to_string(), json!(s));
        }
        if let Some(p) = progress {
            variables.insert("progress".to_string(), json!(p));
        }
        if let Some(sc) = score {
            variables.insert("score".to_string(), json!(sc));
        }

        let body = json!({
            "query": mutation,
            "variables": variables
        });

        let res = self
            .client
            .post(ANILIST_GRAPHQL_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&body)
            .send()
            .await?;

        let json_val: serde_json::Value = res.json().await?;
        let entry_val = &json_val["data"]["SaveMediaListEntry"];
        if entry_val.is_null() {
            return Err(AppError::Parse("SaveMediaListEntry returned null".to_string()));
        }

        let id = entry_val["mediaId"].as_i64().unwrap_or(media_id as i64) as i32;
        let res_progress = entry_val["progress"].as_i64().unwrap_or(0) as i32;
        let res_score = entry_val["score"].as_f64().unwrap_or(0.0);
        let res_status = entry_val["status"].as_str().map(String::from);

        Ok(crate::models::MediaListEntry {
            id,
            progress: res_progress,
            score: res_score,
            status: res_status,
            title: None,
            cover: None,
            format_str: None,
            total_episodes: None,
            average_score: None,
        })
    }

    pub async fn fetch_airing_schedule(
        &self,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<Vec<AiringEntry>, AppError> {
        const AIRING_SCHEDULE_QUERY: &str = r#"
        query ($from: Int, $to: Int, $page: Int) {
            Page(page: $page, perPage: 50) {
                pageInfo {
                    hasNextPage
                }
                airingSchedules(airingAt_greater: $from, airingAt_lesser: $to) {
                    id
                    airingAt
                    episode
                    media {
                        id
                        title {
                            english
                            userPreferred
                            romaji
                        }
                        coverImage {
                            extraLarge
                            large
                        }
                        bannerImage
                        format
                        duration
                        averageScore
                    }
                }
            }
        }
        "#;

        let mut all_entries = Vec::new();
        let mut page = 1;

        loop {
            let body = json!({
                "query": AIRING_SCHEDULE_QUERY,
                "variables": {
                    "from": from_ts,
                    "to": to_ts,
                    "page": page,
                }
            });

            let res = self
                .client
                .post(ANILIST_GRAPHQL_URL)
                .json(&body)
                .send()
                .await?;

            let json: serde_json::Value = res.json().await?;
            let page_data = &json["data"]["Page"];
            let schedules = page_data["airingSchedules"]
                .as_array()
                .ok_or_else(|| AppError::Parse("Failed to parse airingSchedules".to_string()))?;

            for s in schedules {
                let id = s["id"].as_i64().unwrap_or(0);
                let airing_at = s["airingAt"].as_i64().unwrap_or(0);
                let episode = s["episode"].as_i64().unwrap_or(0) as i32;

                let media = &s["media"];
                let media_id = media["id"].as_i64().unwrap_or(0);

                let title_val = &media["title"];
                let media_title = title_val["english"]
                    .as_str()
                    .or_else(|| title_val["userPreferred"].as_str())
                    .or_else(|| title_val["romaji"].as_str())
                    .unwrap_or("Untitled")
                    .to_string();

                let cover_val = &media["coverImage"];
                let cover_image = cover_val["extraLarge"]
                    .as_str()
                    .or_else(|| cover_val["large"].as_str())
                    .map(String::from);

                let banner_image = media["bannerImage"].as_str().map(String::from);
                let format = media["format"].as_str().map(String::from);
                let duration = media["duration"].as_i64().map(|v| v as i32);
                let average_score = media["averageScore"].as_i64().map(|v| v as i32);

                all_entries.push(AiringEntry {
                    id,
                    airing_at,
                    episode,
                    media_id,
                    media_title,
                    cover_image,
                    banner_image,
                    format,
                    duration,
                    average_score,
                });
            }

            let has_next_page = page_data["pageInfo"]["hasNextPage"].as_bool().unwrap_or(false);
            if !has_next_page || page >= 10 {
                break;
            }
            page += 1;
        }

        Ok(all_entries)
    }
}



pub fn parse_media_json(val: &serde_json::Value) -> Result<Media, AppError> {
    let id = val["id"]
        .as_i64()
        .ok_or_else(|| AppError::Parse("Missing media id".to_string()))? as i32;

    let id_mal = val["idMal"].as_i64().map(|v| v as i32);

    let title_val = &val["title"];
    let title = MediaTitle {
        romaji: title_val["romaji"].as_str().map(String::from),
        english: title_val["english"].as_str().map(String::from),
        native: title_val["native"].as_str().map(String::from),
        user_preferred: title_val["userPreferred"].as_str().map(String::from),
    };

    let cover_val = &val["coverImage"];
    let cover_image = CoverImage {
        large: cover_val["large"].as_str().map(String::from),
        extra_large: cover_val["extraLarge"].as_str().map(String::from),
        color: cover_val["color"].as_str().map(String::from),
    };

    let banner_image = val["bannerImage"].as_str().map(String::from);
    let description = val["description"].as_str().map(String::from);
    let format = val["format"].as_str().map(String::from);
    let season = val["season"].as_str().map(String::from);
    let season_year = val["seasonYear"].as_i64().map(|v| v as i32);
    let episodes = val["episodes"].as_i64().map(|v| v as i32);
    let duration = val["duration"].as_i64().map(|v| v as i32);
    let status = val["status"].as_str().map(String::from);
    let average_score = val["averageScore"].as_i64().map(|v| v as i32);
    let mean_score = val["meanScore"].as_i64().map(|v| v as i32);
    let popularity = val["popularity"].as_i64().map(|v| v as i32);
    let favourites = val["favourites"].as_i64().map(|v| v as i32);
    let is_adult = val["isAdult"].as_bool().unwrap_or(false);

    let genres = val["genres"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(Media {
        id,
        id_mal,
        title,
        cover_image,
        banner_image,
        description,
        format,
        season,
        season_year,
        episodes,
        duration,
        status,
        average_score,
        mean_score,
        popularity,
        favourites,
        is_adult,
        genres,
    })
}
