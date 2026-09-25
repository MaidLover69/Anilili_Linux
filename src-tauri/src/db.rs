use crate::error::AppError;
use crate::models::{AiringEntry, HistoryEntry, NotificationPreference, CustomPlaylist, CustomPlaylistItem};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub fn get_app_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::Other("Could not find home directory".to_string()))?;
    let path = home.join(".local").join("share").join("anilili");
    fs::create_dir_all(&path).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(path)
}

pub fn get_config_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::Other("Could not find home directory".to_string()))?;
    let path = home.join(".config").join("anilili");
    fs::create_dir_all(&path).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(path)
}

pub async fn init_db() -> Result<SqlitePool, AppError> {
    let app_dir = get_app_dir()?;
    let db_path = app_dir.join("anilili.db");

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    // Create tables
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

        CREATE TABLE IF NOT EXISTS custom_playlists (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            description TEXT,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS custom_playlist_items (
            id            TEXT PRIMARY KEY,
            playlist_id   TEXT NOT NULL,
            anilist_id    INTEGER NOT NULL,
            mal_id        INTEGER,
            episode_num   REAL NOT NULL,
            episode_title TEXT,
            series_title  TEXT NOT NULL,
            series_cover  TEXT,
            category      TEXT NOT NULL DEFAULT 'sub',
            sort_order    INTEGER NOT NULL DEFAULT 0,
            added_at      INTEGER NOT NULL,
            FOREIGN KEY(playlist_id) REFERENCES custom_playlists(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let columns: Vec<String> = sqlx::query("PRAGMA table_info(watch_history);")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect();

    if columns.contains(&"episode".to_string()) && !columns.contains(&"episode_number".to_string()) {
        let _ = sqlx::query("DROP TABLE watch_history;").execute(&pool).await;
        let _ = sqlx::query(
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
            "#,
        )
        .execute(&pool)
        .await;
    }

    let required_cols = [
        ("episode_number", "REAL NOT NULL DEFAULT 1.0"),
        ("episode_title", "TEXT"),
        ("provider", "TEXT NOT NULL DEFAULT 'auto'"),
        ("category", "TEXT NOT NULL DEFAULT 'sub'"),
        ("position_ms", "INTEGER NOT NULL DEFAULT 0"),
        ("duration_ms", "INTEGER NOT NULL DEFAULT 0"),
        ("updated_at", "INTEGER NOT NULL DEFAULT 0"),
        ("from_remote", "INTEGER NOT NULL DEFAULT 0"),
        ("title", "TEXT NOT NULL DEFAULT ''"),
        ("cover", "TEXT"),
    ];

    for (col_name, col_def) in required_cols {
        if !columns.contains(&col_name.to_string()) {
            let alter_sql = format!("ALTER TABLE watch_history ADD COLUMN {} {};", col_name, col_def);
            let _ = sqlx::query(&alter_sql).execute(&pool).await;
        }
    }

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_updated ON watch_history(updated_at DESC);")
        .execute(&pool)
        .await;

    info!("Database initialized at {}", db_path.display());
    Ok(pool)
}

pub async fn get_continue_watching(pool: &SqlitePool) -> Result<Vec<HistoryEntry>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT anilist_id, episode_number, episode_title, provider, category, position_ms, duration_ms, updated_at, from_remote, title, cover
        FROM watch_history
        WHERE position_ms > 0
        GROUP BY anilist_id
        ORDER BY updated_at DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await?;

    let entries = rows
        .into_iter()
        .map(|r| {
            let anilist_id: i64 = r.get("anilist_id");
            let episode_number: f64 = r.get("episode_number");
            let episode_title: Option<String> = r.get("episode_title");
            let provider: String = r.get("provider");
            let category: String = r.get("category");
            let position_ms: i64 = r.get("position_ms");
            let duration_ms: i64 = r.get("duration_ms");
            let updated_at: i64 = r.get("updated_at");
            let from_remote: i64 = r.get("from_remote");
            let title: String = r.get("title");
            let cover: Option<String> = r.get("cover");

            HistoryEntry {
                anilist_id: anilist_id as i32,
                title,
                cover,
                episode_number,
                episode_title,
                provider,
                category,
                position_ms,
                duration_ms,
                updated_at,
                from_remote: from_remote != 0,
            }
        })
        .collect();

    Ok(entries)
}

pub async fn get_watched_episodes(pool: &SqlitePool, anilist_id: i32) -> Result<Vec<f64>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT episode_number
        FROM watch_history
        WHERE anilist_id = ?1 AND (position_ms >= (duration_ms * 0.85) OR position_ms > 0)
        "#
    )
    .bind(anilist_id)
    .fetch_all(pool)
    .await?;

    let list = rows.into_iter().map(|r| r.get::<f64, _>("episode_number")).collect();
    Ok(list)
}

pub async fn get_episode_progress(
    pool: &SqlitePool,
    anilist_id: i32,
    episode_number: f64,
) -> Result<Option<(i64, i64)>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT position_ms, duration_ms
        FROM watch_history
        WHERE anilist_id = ?1 AND ABS(episode_number - ?2) < 0.01
        ORDER BY updated_at DESC
        LIMIT 1
        "#
    )
    .bind(anilist_id)
    .bind(episode_number)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        let pos: i64 = r.get("position_ms");
        let dur: i64 = r.get("duration_ms");
        Ok(Some((pos, dur)))
    } else {
        Ok(None)
    }
}

pub async fn update_watch_progress(
    pool: &SqlitePool,
    entry: &HistoryEntry,
) -> Result<(), AppError> {
    let from_remote_val = if entry.from_remote { 1 } else { 0 };

    sqlx::query(
        r#"
        INSERT INTO watch_history (anilist_id, episode_number, episode_title, provider, category, position_ms, duration_ms, updated_at, from_remote, title, cover)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(anilist_id) DO UPDATE SET
            episode_number = excluded.episode_number,
            episode_title = excluded.episode_title,
            provider = excluded.provider,
            category = excluded.category,
            position_ms = excluded.position_ms,
            duration_ms = excluded.duration_ms,
            updated_at = excluded.updated_at,
            from_remote = excluded.from_remote,
            title = excluded.title,
            cover = excluded.cover;
        "#
    )
    .bind(entry.anilist_id)
    .bind(entry.episode_number)
    .bind(&entry.episode_title)
    .bind(&entry.provider)
    .bind(&entry.category)
    .bind(entry.position_ms)
    .bind(entry.duration_ms)
    .bind(entry.updated_at)
    .bind(from_remote_val)
    .bind(&entry.title)
    .bind(&entry.cover)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn add_to_watchlist(
    pool: &SqlitePool,
    anilist_id: i32,
    title: &str,
    cover: Option<&str>,
    format: Option<&str>,
    average_score: Option<i32>,
) -> Result<(), AppError> {
    let now = current_unix_timestamp() * 1000;

    sqlx::query(
        r#"
        INSERT OR IGNORE INTO watchlist (anilist_id, title, cover, format, average_score, added_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#
    )
    .bind(anilist_id)
    .bind(title)
    .bind(cover)
    .bind(format)
    .bind(average_score)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_from_watchlist(pool: &SqlitePool, anilist_id: i32) -> Result<(), AppError> {
    sqlx::query("DELETE FROM watchlist WHERE anilist_id = ?1")
        .bind(anilist_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn is_in_watchlist(pool: &SqlitePool, anilist_id: i32) -> Result<bool, AppError> {
    let row = sqlx::query("SELECT anilist_id FROM watchlist WHERE anilist_id = ?1")
        .bind(anilist_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

pub async fn get_watchlist(pool: &SqlitePool) -> Result<Vec<crate::models::WatchlistEntry>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT anilist_id, title, cover, format, average_score, added_at
        FROM watchlist
        ORDER BY added_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let entries = rows
        .into_iter()
        .map(|r| crate::models::WatchlistEntry {
            anilist_id: r.get("anilist_id"),
            title: r.get("title"),
            cover: r.get("cover"),
            format: r.get("format"),
            average_score: r.get("average_score"),
            added_at: r.get("added_at"),
        })
        .collect();

    Ok(entries)
}

pub async fn upsert_library_entry(
    pool: &SqlitePool,
    entry: &crate::models::MediaListEntry,
) -> Result<(), AppError> {
    let now = current_unix_timestamp() * 1000;

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO remote_library (anilist_id, status, progress, score, title, cover, format, episodes, average_score, synced_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#
    )
    .bind(entry.id)
    .bind(&entry.status)
    .bind(entry.progress)
    .bind(entry.score)
    .bind(&entry.title)
    .bind(&entry.cover)
    .bind(&entry.format_str)
    .bind(entry.total_episodes)
    .bind(entry.average_score)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_library_entries(
    pool: &SqlitePool,
    status_filter: Option<String>,
) -> Result<Vec<crate::models::MediaListEntry>, AppError> {
    let rows = if let Some(ref status) = status_filter {
        sqlx::query(
            r#"
            SELECT anilist_id, status, progress, score, title, cover, format, episodes, average_score
            FROM remote_library
            WHERE status = ?1
            ORDER BY title ASC
            "#
        )
        .bind(status)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT anilist_id, status, progress, score, title, cover, format, episodes, average_score
            FROM remote_library
            ORDER BY title ASC
            "#
        )
        .fetch_all(pool)
        .await?
    };

    let entries = rows
        .into_iter()
        .map(|r| crate::models::MediaListEntry {
            id: r.get("anilist_id"),
            status: r.get("status"),
            progress: r.get("progress"),
            score: r.get("score"),
            title: r.get("title"),
            cover: r.get("cover"),
            format_str: r.get("format"),
            total_episodes: r.get("episodes"),
            average_score: r.get("average_score"),
        })
        .collect();

    Ok(entries)
}

pub async fn create_download(
    pool: &SqlitePool,
    record: &crate::models::DownloadRecord,
) -> Result<String, AppError> {
    let now = current_unix_timestamp() * 1000;

    sqlx::query(
        r#"
        INSERT INTO downloads (id, anilist_id, episode_num, episode_title, series_title, series_cover, provider, category, quality, status, progress, file_path, file_size, duration_s, error_msg, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
        "#
    )
    .bind(&record.id)
    .bind(record.anilist_id)
    .bind(record.episode_num)
    .bind(&record.episode_title)
    .bind(&record.series_title)
    .bind(&record.series_cover)
    .bind(&record.provider)
    .bind(&record.category)
    .bind(&record.quality)
    .bind(&record.status)
    .bind(record.progress)
    .bind(&record.file_path)
    .bind(record.file_size)
    .bind(record.duration_s)
    .bind(&record.error_msg)
    .bind(if record.created_at > 0 { record.created_at } else { now })
    .bind(now)
    .execute(pool)
    .await?;

    Ok(record.id.clone())
}

pub async fn update_download_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    progress: Option<f64>,
    file_path: Option<&str>,
    file_size: Option<i64>,
    duration_s: Option<f64>,
    error_msg: Option<&str>,
) -> Result<(), AppError> {
    let now = current_unix_timestamp() * 1000;

    sqlx::query(
        r#"
        UPDATE downloads
        SET status = ?1,
            progress = COALESCE(?2, progress),
            file_path = COALESCE(?3, file_path),
            file_size = COALESCE(?4, file_size),
            duration_s = COALESCE(?5, duration_s),
            error_msg = ?6,
            updated_at = ?7
        WHERE id = ?8
        "#
    )
    .bind(status)
    .bind(progress)
    .bind(file_path)
    .bind(file_size)
    .bind(duration_s)
    .bind(error_msg)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

fn map_download_row(r: sqlx::sqlite::SqliteRow) -> crate::models::DownloadRecord {
    crate::models::DownloadRecord {
        id: r.get("id"),
        anilist_id: r.get("anilist_id"),
        episode_num: r.get("episode_num"),
        episode_title: r.get("episode_title"),
        series_title: r.get("series_title"),
        series_cover: r.get("series_cover"),
        provider: r.get("provider"),
        category: r.get("category"),
        quality: r.get("quality"),
        status: r.get("status"),
        progress: r.get("progress"),
        file_path: r.get("file_path"),
        file_size: r.get("file_size"),
        duration_s: r.get("duration_s"),
        error_msg: r.get("error_msg"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}

pub async fn get_downloads(
    pool: &SqlitePool,
    anilist_id: i64,
) -> Result<Vec<crate::models::DownloadRecord>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, anilist_id, episode_num, episode_title, series_title, series_cover, provider, category, quality, status, progress, file_path, file_size, duration_s, error_msg, created_at, updated_at
        FROM downloads
        WHERE anilist_id = ?1
        ORDER BY episode_num ASC
        "#
    )
    .bind(anilist_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(map_download_row).collect())
}

pub async fn get_all_downloads(
    pool: &SqlitePool,
) -> Result<Vec<crate::models::DownloadRecord>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, anilist_id, episode_num, episode_title, series_title, series_cover, provider, category, quality, status, progress, file_path, file_size, duration_s, error_msg, created_at, updated_at
        FROM downloads
        ORDER BY updated_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(map_download_row).collect())
}

pub async fn delete_download(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM downloads WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_download_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<crate::models::DownloadRecord>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT id, anilist_id, episode_num, episode_title, series_title, series_cover, provider, category, quality, status, progress, file_path, file_size, duration_s, error_msg, created_at, updated_at
        FROM downloads
        WHERE id = ?1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_download_row))
}

pub fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub async fn save_notification_preference(
    pool: &SqlitePool,
    pref: &NotificationPreference,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO notification_preferences (media_id, enabled, media_title, cover_image)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(media_id) DO UPDATE SET
            enabled = excluded.enabled,
            media_title = excluded.media_title,
            cover_image = excluded.cover_image
        "#,
    )
    .bind(pref.media_id)
    .bind(if pref.enabled { 1i32 } else { 0i32 })
    .bind(&pref.media_title)
    .bind(&pref.cover_image)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_notification_preference(
    pool: &SqlitePool,
    media_id: i64,
) -> Result<Option<NotificationPreference>, AppError> {
    let row = sqlx::query(
        "SELECT media_id, enabled, media_title, cover_image FROM notification_preferences WHERE media_id = ?1",
    )
    .bind(media_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| NotificationPreference {
        media_id: r.get("media_id"),
        enabled: r.get::<i32, _>("enabled") != 0,
        media_title: r.get("media_title"),
        cover_image: r.get("cover_image"),
    }))
}

pub async fn list_notification_preferences(
    pool: &SqlitePool,
) -> Result<Vec<NotificationPreference>, AppError> {
    let rows = sqlx::query(
        "SELECT media_id, enabled, media_title, cover_image FROM notification_preferences WHERE enabled = 1",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| NotificationPreference {
            media_id: r.get("media_id"),
            enabled: r.get::<i32, _>("enabled") != 0,
            media_title: r.get("media_title"),
            cover_image: r.get("cover_image"),
        })
        .collect())
}

pub async fn get_cached_schedule(
    pool: &SqlitePool,
    from_ts: i64,
    to_ts: i64,
) -> Result<Option<Vec<AiringEntry>>, AppError> {
    let now = current_unix_timestamp();
    let row = sqlx::query(
        "SELECT data, cached_at FROM schedule_cache WHERE from_ts = ?1 AND to_ts = ?2",
    )
    .bind(from_ts)
    .bind(to_ts)
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        let cached_at: i64 = r.get("cached_at");
        if now - cached_at < 1800 {
            let data_str: String = r.get("data");
            if let Ok(entries) = serde_json::from_str::<Vec<AiringEntry>>(&data_str) {
                return Ok(Some(entries));
            }
        }
    }

    Ok(None)
}

pub async fn cache_schedule(
    pool: &SqlitePool,
    from_ts: i64,
    to_ts: i64,
    entries: &[AiringEntry],
) -> Result<(), AppError> {
    let now = current_unix_timestamp();
    let json_data = serde_json::to_string(entries)
        .map_err(|e| AppError::Other(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO schedule_cache (from_ts, to_ts, data, cached_at)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(from_ts, to_ts) DO UPDATE SET
            data = excluded.data,
            cached_at = excluded.cached_at
        "#,
    )
    .bind(from_ts)
    .bind(to_ts)
    .bind(json_data)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_cached_filler_episodes(
    pool: &SqlitePool,
    mal_id: i32,
) -> Result<Option<Vec<i32>>, AppError> {
    let now = current_unix_timestamp();
    let row = sqlx::query("SELECT filler_json, cached_at FROM filler_cache WHERE mal_id = ?1")
        .bind(mal_id)
        .fetch_optional(pool)
        .await?;

    if let Some(r) = row {
        let cached_at: i64 = r.get("cached_at");
        if now - cached_at < 604800 {
            let json_str: String = r.get("filler_json");
            if let Ok(episodes) = serde_json::from_str::<Vec<i32>>(&json_str) {
                return Ok(Some(episodes));
            }
        }
    }

    Ok(None)
}

pub async fn cache_filler_episodes(
    pool: &SqlitePool,
    mal_id: i32,
    episodes: &[i32],
) -> Result<(), AppError> {
    let now = current_unix_timestamp();
    let json_str = serde_json::to_string(episodes)
        .map_err(|e| AppError::Other(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO filler_cache (mal_id, filler_json, cached_at)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(mal_id) DO UPDATE SET
            filler_json = excluded.filler_json,
            cached_at = excluded.cached_at
        "#,
    )
    .bind(mal_id)
    .bind(json_str)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_custom_playlist(
    pool: &SqlitePool,
    name: &str,
    description: Option<&str>,
) -> Result<CustomPlaylist, AppError> {
    let now = current_unix_timestamp();
    let id = format!("pl-{}", now);
    sqlx::query(
        r#"
        INSERT INTO custom_playlists (id, name, description, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(description)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(CustomPlaylist {
        id,
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: now,
        updated_at: now,
        item_count: 0,
        covers: vec![],
    })
}

pub async fn list_custom_playlists(
    pool: &SqlitePool,
) -> Result<Vec<CustomPlaylist>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            p.id, p.name, p.description, p.created_at, p.updated_at,
            COUNT(i.id) as item_count,
            GROUP_CONCAT(DISTINCT i.series_cover) as covers_str
        FROM custom_playlists p
        LEFT JOIN custom_playlist_items i ON p.id = i.playlist_id
        GROUP BY p.id
        ORDER BY p.updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut playlists = Vec::new();
    for r in rows {
        let covers_str: Option<String> = r.get("covers_str");
        let covers = covers_str
            .map(|s| {
                s.split(',')
                    .filter(|c| !c.is_empty())
                    .take(4)
                    .map(|c| c.to_string())
                    .collect()
            })
            .unwrap_or_default();

        playlists.push(CustomPlaylist {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            item_count: r.get("item_count"),
            covers,
        });
    }

    Ok(playlists)
}

pub async fn get_custom_playlist_items(
    pool: &SqlitePool,
    playlist_id: &str,
) -> Result<Vec<CustomPlaylistItem>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, playlist_id, anilist_id, mal_id, episode_num, episode_title,
               series_title, series_cover, category, sort_order, added_at
        FROM custom_playlist_items
        WHERE playlist_id = ?1
        ORDER BY sort_order ASC, added_at ASC
        "#,
    )
    .bind(playlist_id)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| CustomPlaylistItem {
            id: r.get("id"),
            playlist_id: r.get("playlist_id"),
            anilist_id: r.get("anilist_id"),
            mal_id: r.get("mal_id"),
            episode_num: r.get("episode_num"),
            episode_title: r.get("episode_title"),
            series_title: r.get("series_title"),
            series_cover: r.get("series_cover"),
            category: r.get("category"),
            sort_order: r.get("sort_order"),
            added_at: r.get("added_at"),
        })
        .collect();

    Ok(items)
}

pub async fn add_item_to_custom_playlist(
    pool: &SqlitePool,
    item: CustomPlaylistItem,
) -> Result<(), AppError> {
    let now = current_unix_timestamp();
    sqlx::query(
        r#"
        INSERT INTO custom_playlist_items 
        (id, playlist_id, anilist_id, mal_id, episode_num, episode_title, series_title, series_cover, category, sort_order, added_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
    )
    .bind(&item.id)
    .bind(&item.playlist_id)
    .bind(item.anilist_id)
    .bind(item.mal_id)
    .bind(item.episode_num)
    .bind(&item.episode_title)
    .bind(&item.series_title)
    .bind(&item.series_cover)
    .bind(&item.category)
    .bind(item.sort_order)
    .bind(item.added_at)
    .execute(pool)
    .await?;

    let _ = sqlx::query("UPDATE custom_playlists SET updated_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(&item.playlist_id)
        .execute(pool)
        .await;

    Ok(())
}

pub async fn remove_item_from_custom_playlist(
    pool: &SqlitePool,
    item_id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM custom_playlist_items WHERE id = ?1")
        .bind(item_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_custom_playlist(
    pool: &SqlitePool,
    playlist_id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM custom_playlist_items WHERE playlist_id = ?1")
        .bind(playlist_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM custom_playlists WHERE id = ?1")
        .bind(playlist_id)
        .execute(pool)
        .await?;
    Ok(())
}

