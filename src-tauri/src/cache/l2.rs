use crate::error::AppError;
use sqlx::{Row, SqlitePool};

pub async fn init_cache_db(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cache_entries (
            cache_key   TEXT PRIMARY KEY,
            data        TEXT NOT NULL,
            expires_at  INTEGER NOT NULL,
            created_at  INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_cache_expires ON cache_entries(expires_at);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_cache_entry(pool: &SqlitePool, key: &str) -> Result<Option<String>, AppError> {
    let now = chrono_now_ms();
    let row = sqlx::query(
        r#"
        SELECT data FROM cache_entries
        WHERE cache_key = ?1 AND expires_at > ?2
        "#,
    )
    .bind(key)
    .bind(now)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.get::<String, _>("data")))
}

pub async fn set_cache_entry(
    pool: &SqlitePool,
    key: &str,
    data: &str,
    ttl_secs: i64,
) -> Result<(), AppError> {
    let now = chrono_now_ms();
    let expires = now + (ttl_secs * 1000);

    sqlx::query(
        r#"
        INSERT INTO cache_entries (cache_key, data, expires_at, created_at)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(cache_key) DO UPDATE SET
            data = excluded.data,
            expires_at = excluded.expires_at,
            created_at = excluded.created_at
        "#,
    )
    .bind(key)
    .bind(data)
    .bind(expires)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

fn chrono_now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
