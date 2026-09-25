use anilili_lib::cache::L1Cache;
use anilili_lib::models::{CoverImage, Media, MediaTitle, SourcesResult, StreamItem};

#[tokio::test]
async fn test_l1_cache() {
    let cache = L1Cache::<String, String>::new(10, 60);
    cache.insert("key1".to_string(), "val1".to_string()).await;
    assert_eq!(cache.get(&"key1".to_string()).await, Some("val1".to_string()));
    assert_eq!(cache.get(&"unknown".to_string()).await, None);
}

#[test]
fn test_media_model_serialization() {
    let media = Media {
        id: 1,
        id_mal: Some(10),
        title: MediaTitle {
            romaji: Some("Cowboy Bebop".to_string()),
            english: Some("Cowboy Bebop".to_string()),
            native: None,
            user_preferred: Some("Cowboy Bebop".to_string()),
        },
        cover_image: CoverImage {
            extra_large: Some("https://example.com/cover.jpg".to_string()),
            large: None,
            color: None,
        },
        banner_image: None,
        description: Some("Space western".to_string()),
        format: Some("TV".to_string()),
        status: Some("FINISHED".to_string()),
        episodes: Some(26),
        duration: Some(24),
        season: Some("SPRING".to_string()),
        season_year: Some(1998),
        average_score: Some(89),
        mean_score: Some(89),
        popularity: Some(100000),
        favourites: Some(50000),
        genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
        is_adult: false,
        ..Default::default()
    };

    let json_str = serde_json::to_string(&media).expect("serialize media");
    let deserialized: Media = serde_json::from_str(&json_str).expect("deserialize media");
    assert_eq!(deserialized.id, 1);
    assert_eq!(deserialized.title.english, Some("Cowboy Bebop".to_string()));
    assert_eq!(deserialized.display_title(), "Cowboy Bebop");
}

#[test]
fn test_stream_item_and_sources() {
    let stream = StreamItem {
        url: "https://example.com/master.m3u8".to_string(),
        stream_type: "hls".to_string(),
        quality: Some("1080p".to_string()),
        audio: Some("sub".to_string()),
        subtitle_variant: None,
        referer: Some("https://example.com/".to_string()),
        is_active: true,
        origin: Some("https://example.com".to_string()),
        headers: None,
    };

    let sources = SourcesResult {
        streams: vec![stream.clone()],
        subtitles: vec![],
        skip: None,
    };

    assert_eq!(sources.streams.len(), 1);
    assert_eq!(sources.streams[0].url, "https://example.com/master.m3u8");
}

#[tokio::test]
async fn test_airing_schedule_fetch() {
    use anilili_lib::clients::anilist::AniListClient;
    let client = AniListClient::new(reqwest::Client::new());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let from_ts = now - 86400;
    let to_ts = now + 86400 * 7;
    let result = client.fetch_airing_schedule(from_ts, to_ts).await;
    match result {
        Ok(entries) => {
            println!("Fetched {} airing entries successfully", entries.len());
        }
        Err(e) => {
            println!("Network skipped or unavailable in test runner: {:?}", e);
        }
    }
}

#[test]
fn test_anime_themes_model() {
    use anilili_lib::clients::animethemes::AnimeThemeEntry;
    let theme = AnimeThemeEntry {
        theme_type: "OP".to_string(),
        sequence: Some(1),
        slug: "OP1".to_string(),
        song_title: Some("Yuusha".to_string()),
        artist_name: Some("YOASOBI".to_string()),
        episodes: Some("1-16".to_string()),
        video_url: Some("https://v.animethemes.moe/SousouNoFrieren-OP1.webm".to_string()),
        audio_url: Some("https://a.animethemes.moe/SousouNoFrieren-OP1.ogg".to_string()),
        resolution: Some(1080),
    };
    assert_eq!(theme.slug, "OP1");
    assert_eq!(theme.resolution, Some(1080));
}

#[test]
fn test_local_anime_file_model() {
    use anilili_lib::commands::local_scanner::LocalAnimeFile;
    let file = LocalAnimeFile {
        file_path: "/videos/Frieren - 01.mkv".to_string(),
        file_name: "Frieren - 01.mkv".to_string(),
        file_size: 1024 * 1024 * 500,
        parsed_title: "Frieren".to_string(),
        parsed_episode: Some(1.0),
    };
    assert_eq!(file.parsed_title, "Frieren");
    assert_eq!(file.parsed_episode, Some(1.0));
}
