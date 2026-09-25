use anilili_lib::db;
use anilili_lib::models::{DownloadRecord, HistoryEntry, NotificationPreference};
use sqlx::sqlite::SqlitePoolOptions;

async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory SQLite");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS watch_history (
            anilist_id      INTEGER PRIMARY KEY,
            episode_number  REAL NOT NULL DEFAULT 1.0,
            episode_title   TEXT,
            provider        TEXT NOT NULL DEFAULT 'auto',
            category        TEXT NOT NULL DEFAULT 'sub',
            position_ms     INTEGER NOT NULL DEFAULT 0,
            duration_ms     INTEGER NOT NULL DEFAULT 0,
            updated_at      INTEGER NOT NULL DEFAULT 0,
            from_remote     INTEGER NOT NULL DEFAULT 0,
            title           TEXT NOT NULL DEFAULT '',
            cover           TEXT
        );

        CREATE TABLE IF NOT EXISTS watchlist (
            anilist_id      INTEGER PRIMARY KEY,
            title           TEXT NOT NULL,
            cover           TEXT,
            format          TEXT,
            average_score   INTEGER,
            added_at        INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS remote_library (
            anilist_id      INTEGER PRIMARY KEY,
            status          TEXT,
            progress        INTEGER NOT NULL DEFAULT 0,
            score           REAL NOT NULL DEFAULT 0.0,
            title           TEXT,
            cover           TEXT,
            format          TEXT,
            episodes        INTEGER,
            average_score   INTEGER,
            synced_at       INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS downloads (
            id            TEXT PRIMARY KEY,
            anilist_id    INTEGER NOT NULL,
            episode_num   REAL NOT NULL,
            episode_title TEXT,
            series_title  TEXT NOT NULL,
            series_cover  TEXT,
            provider      TEXT NOT NULL,
            category      TEXT NOT NULL,
            quality       TEXT NOT NULL,
            status        TEXT NOT NULL DEFAULT 'QUEUED',
            progress      REAL DEFAULT 0.0,
            file_path     TEXT,
            file_size     INTEGER,
            duration_s    REAL,
            error_msg     TEXT,
            created_at    INTEGER NOT NULL,
            updated_at    INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notification_preferences (
            media_id      INTEGER PRIMARY KEY,
            enabled       INTEGER NOT NULL DEFAULT 1,
            media_title   TEXT NOT NULL,
            cover_image   TEXT
        );

        CREATE TABLE IF NOT EXISTS schedule_cache (
            from_ts     INTEGER NOT NULL,
            to_ts       INTEGER NOT NULL,
            data        TEXT NOT NULL,
            cached_at   INTEGER NOT NULL,
            PRIMARY KEY (from_ts, to_ts)
        );

        CREATE TABLE IF NOT EXISTS filler_cache (
            mal_id      INTEGER PRIMARY KEY,
            filler_json TEXT NOT NULL,
            cached_at   INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to initialize test DB schema");

    pool
}

#[tokio::test]
async fn test_watch_history_and_progress() {
    let pool = setup_test_db().await;

    let entry = HistoryEntry {
        anilist_id: 101,
        title: "Attack on Titan".to_string(),
        cover: Some("https://example.com/aot.jpg".to_string()),
        episode_number: 1.0,
        episode_title: Some("To You, in 2000 Years".to_string()),
        provider: "Senshi".to_string(),
        category: "sub".to_string(),
        position_ms: 120000,
        duration_ms: 1440000,
        updated_at: 1000,
        from_remote: false,
    };

    db::update_watch_progress(&pool, &entry).await.expect("update progress");

    let continue_list = db::get_continue_watching(&pool).await.expect("get continue watching");
    assert_eq!(continue_list.len(), 1);
    assert_eq!(continue_list[0].title, "Attack on Titan");
    assert_eq!(continue_list[0].position_ms, 120000);

    let progress = db::get_episode_progress(&pool, 101, 1.0)
        .await
        .expect("get progress")
        .expect("progress exists");
    assert_eq!(progress.0, 120000);
    assert_eq!(progress.1, 1440000);

    let watched = db::get_watched_episodes(&pool, 101).await.expect("get watched");
    assert_eq!(watched, vec![1.0]);

    // Test Deduplication when watching Episode 2
    let mut entry_ep2 = entry.clone();
    entry_ep2.episode_number = 2.0;
    entry_ep2.episode_title = Some("That Day".to_string());
    entry_ep2.position_ms = 450000;
    entry_ep2.updated_at = 2000;
    db::update_watch_progress(&pool, &entry_ep2).await.expect("update progress ep 2");

    let continue_list2 = db::get_continue_watching(&pool).await.expect("get continue watching ep 2");
    assert_eq!(continue_list2.len(), 1, "Watch history must be deduplicated to 1 entry per anime");
    assert_eq!(continue_list2[0].episode_number, 2.0);
    assert_eq!(continue_list2[0].position_ms, 450000);
}

#[tokio::test]
async fn test_watchlist_operations() {
    let pool = setup_test_db().await;

    assert_eq!(db::is_in_watchlist(&pool, 202).await.expect("check"), false);

    db::add_to_watchlist(
        &pool,
        202,
        "Steins;Gate",
        Some("https://example.com/sg.jpg"),
        Some("TV"),
        Some(91),
    )
    .await
    .expect("add to watchlist");

    assert_eq!(db::is_in_watchlist(&pool, 202).await.expect("check"), true);

    let list = db::get_watchlist(&pool).await.expect("get list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].title, "Steins;Gate");
    assert_eq!(list[0].average_score, Some(91));

    db::remove_from_watchlist(&pool, 202).await.expect("remove");
    assert_eq!(db::is_in_watchlist(&pool, 202).await.expect("check"), false);
}

#[tokio::test]
async fn test_downloads_crud() {
    let pool = setup_test_db().await;

    let rec = DownloadRecord {
        id: "dl-1".to_string(),
        anilist_id: 303,
        episode_num: 5.0,
        episode_title: Some("Episode 5".to_string()),
        series_title: "Frieren".to_string(),
        series_cover: None,
        provider: "Senshi".to_string(),
        category: "sub".to_string(),
        quality: "1080p".to_string(),
        status: "QUEUED".to_string(),
        progress: 0.0,
        file_path: None,
        file_size: None,
        duration_s: None,
        error_msg: None,
        created_at: 1000,
        updated_at: 1000,
    };

    let id = db::create_download(&pool, &rec).await.expect("create dl");
    assert_eq!(id, "dl-1");

    let dls = db::get_downloads(&pool, 303).await.expect("get dls");
    assert_eq!(dls.len(), 1);
    assert_eq!(dls[0].status, "QUEUED");

    db::update_download_status(
        &pool,
        "dl-1",
        "COMPLETED",
        Some(100.0),
        Some("/home/user/frieren_ep5.mp4"),
        Some(350_000_000),
        Some(1420.0),
        None,
    )
    .await
    .expect("update status");

    let updated = db::get_download_by_id(&pool, "dl-1")
        .await
        .expect("fetch dl")
        .expect("exists");
    assert_eq!(updated.status, "COMPLETED");
    assert_eq!(updated.progress, 100.0);
    assert_eq!(updated.file_path, Some("/home/user/frieren_ep5.mp4".to_string()));

    db::delete_download(&pool, "dl-1").await.expect("delete");
    let after_del = db::get_download_by_id(&pool, "dl-1").await.expect("fetch");
    assert!(after_del.is_none());
}

#[tokio::test]
async fn test_notification_and_cache() {
    let pool = setup_test_db().await;

    let pref = NotificationPreference {
        media_id: 404,
        enabled: true,
        media_title: "Jujutsu Kaisen".to_string(),
        cover_image: None,
    };

    db::save_notification_preference(&pool, &pref).await.expect("save pref");
    let fetched = db::get_notification_preference(&pool, 404)
        .await
        .expect("get pref")
        .expect("exists");
    assert_eq!(fetched.enabled, true);
    assert_eq!(fetched.media_title, "Jujutsu Kaisen");

    // Filler cache
    db::cache_filler_episodes(&pool, 505, &[10, 11, 12, 13]).await.expect("cache filler");
    let fillers = db::get_cached_filler_episodes(&pool, 505)
        .await
        .expect("get fillers")
        .expect("fillers exist");
    assert_eq!(fillers, vec![10, 11, 12, 13]);
}
