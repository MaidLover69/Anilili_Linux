use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Provider error: {provider} — {message}")]
    Provider {
        provider: &'static str,
        message: String,
    },

    #[error("CAPTCHA required")]
    CaptchaRequired,

    #[error("No sources found")]
    NoSourcesFound,

    #[error("Rate limited — retry after {retry_after_s}s")]
    RateLimited { retry_after_s: u64 },

    #[error("Auth required")]
    AuthRequired,

    #[error("Other error: {0}")]
    Other(String),
}
