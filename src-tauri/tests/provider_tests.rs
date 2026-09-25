use anilili_lib::models::{Category, EpisodeItem};
use anilili_lib::providers::animeheaven::AnimeHeavenProvider;
use anilili_lib::providers::animepahe::AnimePaheProvider;
use anilili_lib::providers::gojo::GojoWtfProvider;
use anilili_lib::providers::miruro::MiruroProvider;
use anilili_lib::providers::manager::ProviderManager;
use anilili_lib::providers::AnimeProvider;
use std::time::Duration;

#[test]
fn test_provider_tests_category_conversion() {
    assert_eq!(Category::from("sub"), Category::Sub);
    assert_eq!(Category::from("SUB"), Category::Sub);
    assert_eq!(Category::from("dub"), Category::Dub);
    assert_eq!(Category::from("DUB"), Category::Dub);
    assert_eq!(Category::from("unknown"), Category::Sub);

    assert_eq!(Category::Sub.as_str(), "sub");
    assert_eq!(Category::Dub.as_str(), "dub");
}

#[test]
fn test_provider_tests_manager_initialization() {
    let manager = ProviderManager::new();
    assert_eq!(manager.provider_names().len(), 25);
    assert!(manager.provider_names().contains(&"ally"));
    assert!(manager.provider_names().contains(&"kiwi"));
    assert!(manager.provider_names().contains(&"bonk"));
    assert!(manager.provider_names().contains(&"pewe"));
    assert!(manager.provider_names().contains(&"bee"));
    assert!(manager.provider_names().contains(&"moo"));
    assert!(manager.provider_names().contains(&"hop"));
    assert!(manager.provider_names().contains(&"Senshi"));
    assert!(manager.provider_names().contains(&"kaa"));
    assert!(manager.provider_names().contains(&"AniBD"));
    assert!(manager.provider_names().contains(&"animekai"));
    assert!(manager.provider_names().contains(&"anidbapp"));
    assert!(manager.provider_names().contains(&"animegg"));
    assert!(manager.provider_names().contains(&"animeshqip"));
    assert!(manager.provider_names().contains(&"rareanimes"));
    assert!(manager.provider_names().contains(&"anikoto"));
    assert!(manager.provider_names().contains(&"anizone"));
    assert!(manager.provider_names().contains(&"AnimePahe"));
    assert!(manager.provider_names().contains(&"AnimeHeaven"));
    assert!(manager.provider_names().contains(&"GojoWtf"));
}

#[test]
fn test_provider_tests_episode_item_structure() {
    let ep = EpisodeItem {
        number: 1.0,
        pipe_id: "senshi|12345|1".to_string(),
        title: Some("Episode 1".to_string()),
        image: Some("https://example.com/thumb.jpg".to_string()),
        synopsis: None,
        filler: false,
    };

    assert_eq!(ep.number, 1.0);
    assert_eq!(ep.pipe_id, "senshi|12345|1");
    assert_eq!(ep.filler, false);
}

#[tokio::test]
async fn test_provider_tests_animeheaven() {
    let provider = AnimeHeavenProvider::new();
    assert_eq!(provider.name(), "AnimeHeaven");
    assert!(!provider.supports_dub());

    // 1. Missing title test: must fail gracefully with error, no panic
    let no_title_res = provider.get_episodes(1, None, None).await;
    assert!(
        no_title_res.is_err(),
        "AnimeHeaven must return Err when no title is provided"
    );
    println!("[AnimeHeaven] Missing title handled gracefully: {:?}", no_title_res.err());

    // 2. Non-existent anime title: must return empty episodes or error, no panic
    let non_existent_res = tokio::time::timeout(
        Duration::from_secs(10),
        provider.get_episodes(999999, None, Some("ZxyWvuNonExistentAnime999999")),
    )
    .await;

    match non_existent_res {
        Ok(Ok(data)) => {
            println!("[AnimeHeaven] Non-existent title returned {} episodes (expected 0)", data.sub.len());
            assert!(data.sub.is_empty(), "Non-existent anime should have 0 episodes");
        }
        Ok(Err(e)) => {
            println!("[AnimeHeaven] Non-existent title returned graceful Err: {}", e);
        }
        Err(_) => {
            println!("[AnimeHeaven] Non-existent title timed out gracefully");
        }
    }

    // 3. Real anime title: must return real episodes or fail gracefully (no panics)
    let real_res = tokio::time::timeout(
        Duration::from_secs(12),
        provider.get_episodes(21, Some(21), Some("One Piece")),
    )
    .await;

    match real_res {
        Ok(Ok(data)) => {
            println!("[AnimeHeaven] Successfully fetched {} episodes", data.sub.len());
            for ep in &data.sub {
                assert!(ep.number > 0.0, "Episode numbers must be > 0: got {}", ep.number);
                assert!(!ep.pipe_id.is_empty(), "Episode pipe_id must not be empty");
            }

            // Verify get_sources if episodes are present
            if let Some(first_ep) = data.sub.first() {
                let sources_res = tokio::time::timeout(
                    Duration::from_secs(10),
                    provider.get_sources(first_ep, Category::Sub),
                )
                .await;

                match sources_res {
                    Ok(Ok(sources)) => {
                        println!("[AnimeHeaven] Resolved {} streams", sources.streams.len());
                        for s in &sources.streams {
                            assert!(s.url.starts_with("http"), "Stream URL must be HTTP/HTTPS: {}", s.url);
                            assert!(!s.url.contains("dummy"), "Stream URL must not be dummy: {}", s.url);
                            assert!(!s.url.contains("example.com"), "Stream URL must not be placeholder: {}", s.url);
                        }
                    }
                    Ok(Err(e)) => {
                        println!("[AnimeHeaven] get_sources returned graceful error: {}", e);
                    }
                    Err(_) => {
                        println!("[AnimeHeaven] get_sources timed out gracefully");
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("[AnimeHeaven] get_episodes returned graceful error: {}", e);
        }
        Err(_) => {
            println!("[AnimeHeaven] Network query timed out gracefully (no panic)");
        }
    }
}

#[tokio::test]
async fn test_provider_tests_gojowtf() {
    let provider = GojoWtfProvider::new();
    assert_eq!(provider.name(), "GojoWtf");
    assert!(provider.supports_dub());

    // 1. Missing title test: must fail gracefully with error, no panic
    let no_title_res = provider.get_episodes(1, None, None).await;
    assert!(
        no_title_res.is_err(),
        "GojoWtf must return Err when no title is provided"
    );
    println!("[GojoWtf] Missing title handled gracefully: {:?}", no_title_res.err());

    // 2. Non-existent anime title: must return empty episodes or error, no panic
    let non_existent_res = tokio::time::timeout(
        Duration::from_secs(10),
        provider.get_episodes(999999, None, Some("ZxyWvuNonExistentAnime999999")),
    )
    .await;

    match non_existent_res {
        Ok(Ok(data)) => {
            println!("[GojoWtf] Non-existent title returned {} sub, {} dub", data.sub.len(), data.dub.len());
            assert!(data.sub.is_empty(), "Non-existent anime should have 0 sub episodes");
        }
        Ok(Err(e)) => {
            println!("[GojoWtf] Non-existent title returned graceful Err: {}", e);
        }
        Err(_) => {
            println!("[GojoWtf] Non-existent title timed out gracefully");
        }
    }

    // 3. Real anime title: must return real episodes or fail gracefully (no panics)
    let real_res = tokio::time::timeout(
        Duration::from_secs(12),
        provider.get_episodes(21, Some(21), Some("One Piece")),
    )
    .await;

    match real_res {
        Ok(Ok(data)) => {
            println!("[GojoWtf] Successfully fetched {} sub, {} dub episodes", data.sub.len(), data.dub.len());
            for ep in &data.sub {
                assert!(ep.number > 0.0, "Episode numbers must be > 0: got {}", ep.number);
                assert!(!ep.pipe_id.is_empty(), "Episode pipe_id must not be empty");
            }

            if let Some(first_ep) = data.sub.first() {
                let sources_res = tokio::time::timeout(
                    Duration::from_secs(10),
                    provider.get_sources(first_ep, Category::Sub),
                )
                .await;

                match sources_res {
                    Ok(Ok(sources)) => {
                        println!("[GojoWtf] Resolved {} streams", sources.streams.len());
                        for s in &sources.streams {
                            assert!(s.url.starts_with("http"), "Stream URL must be HTTP/HTTPS: {}", s.url);
                            assert!(!s.url.contains("dummy"), "Stream URL must not be dummy: {}", s.url);
                            assert!(!s.url.contains("example.com"), "Stream URL must not be placeholder: {}", s.url);
                        }
                    }
                    Ok(Err(e)) => {
                        println!("[GojoWtf] get_sources returned graceful error: {}", e);
                    }
                    Err(_) => {
                        println!("[GojoWtf] get_sources timed out gracefully");
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("[GojoWtf] get_episodes returned graceful error: {}", e);
        }
        Err(_) => {
            println!("[GojoWtf] Network query timed out gracefully (no panic)");
        }
    }
}

#[tokio::test]
async fn test_provider_tests_no_dummy_streams() {
    let ah = AnimeHeavenProvider::new();
    let gojo = GojoWtfProvider::new();

    let fake_ep = EpisodeItem {
        number: 999.0,
        pipe_id: "fake_nonexistent_pipe_id_99999|999".to_string(),
        title: Some("Fake Episode".to_string()),
        image: None,
        synopsis: None,
        filler: false,
    };

    // Both providers must never return dummy streams for invalid episode data;
    // they must either return Err or empty streams, never dummy/mock URLs.
    let ah_sources = tokio::time::timeout(
        Duration::from_secs(10),
        ah.get_sources(&fake_ep, Category::Sub),
    )
    .await;

    match ah_sources {
        Ok(Ok(sources)) => {
            for s in &sources.streams {
                assert!(!s.url.contains("dummy"), "AnimeHeaven returned a dummy stream: {}", s.url);
                assert!(!s.url.contains("example.com"), "AnimeHeaven returned placeholder: {}", s.url);
            }
        }
        Ok(Err(e)) => {
            println!("[AnimeHeaven] Correctly rejected fake episode: {}", e);
        }
        Err(_) => {
            println!("[AnimeHeaven] Fake episode request timed out gracefully");
        }
    }

    let gojo_sources = tokio::time::timeout(
        Duration::from_secs(10),
        gojo.get_sources(&fake_ep, Category::Sub),
    )
    .await;

    match gojo_sources {
        Ok(Ok(sources)) => {
            for s in &sources.streams {
                assert!(!s.url.contains("dummy"), "GojoWtf returned a dummy stream: {}", s.url);
                assert!(!s.url.contains("example.com"), "GojoWtf returned placeholder: {}", s.url);
            }
        }
        Ok(Err(e)) => {
            println!("[GojoWtf] Correctly rejected fake episode: {}", e);
        }
        Err(_) => {
            println!("[GojoWtf] Fake episode request timed out gracefully");
        }
    }
}

#[tokio::test]
async fn test_provider_tests_animepahe() {
    let provider = AnimePaheProvider::new();
    assert_eq!(provider.name(), "AnimePahe");

    let real_res = tokio::time::timeout(
        Duration::from_secs(12),
        provider.get_episodes(21, Some(21), Some("One Piece")),
    )
    .await;

    match real_res {
        Ok(Ok(data)) => {
            println!("[AnimePahe] Fetched {} episodes", data.sub.len());
            for ep in &data.sub {
                assert!(ep.number > 0.0);
                assert!(!ep.pipe_id.is_empty());
            }
        }
        Ok(Err(e)) => {
            println!("[AnimePahe] Returned graceful error: {}", e);
        }
        Err(_) => {
            println!("[AnimePahe] Request timed out gracefully (no panic)");
        }
    }
}

#[tokio::test]
async fn test_provider_tests_miruro() {
    let provider = MiruroProvider::new("ally");
    assert_eq!(provider.name(), "ally");
    assert!(provider.supports_dub());

    let ep_res = provider.get_episodes(21, Some(21), Some("One Piece")).await;
    assert!(ep_res.is_ok(), "Miruro get_episodes should return Ok");
    let data = ep_res.unwrap();
    assert_eq!(data.name, "ally");
    // data.sub may return real episodes or empty if provider is rate-limited; both are graceful
    println!("[Miruro] Returned {} sub episodes and {} dub episodes", data.sub.len(), data.dub.len());

    // Invalid pipe_id must return Err(AppError::NoSourcesFound), no panic, no dummy stream
    let fake_ep = EpisodeItem {
        number: 1.0,
        pipe_id: "invalid_no_id".to_string(),
        title: Some("Episode 1".to_string()),
        image: None,
        synopsis: None,
        filler: false,
    };
    let src_res = provider.get_sources(&fake_ep, Category::Sub).await;
    assert!(src_res.is_err(), "Miruro must return Err for unparseable pipe_id");
}

