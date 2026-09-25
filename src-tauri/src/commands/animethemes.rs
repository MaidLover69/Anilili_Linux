use crate::clients::animethemes::{AnimeThemeEntry, AnimeThemesClient};
use once_cell::sync::Lazy;

static ANIMETHEMES_CLIENT: Lazy<AnimeThemesClient> = Lazy::new(AnimeThemesClient::new);

#[tauri::command]
pub async fn fetch_anime_themes(mal_id: i64) -> Result<Vec<AnimeThemeEntry>, String> {
    ANIMETHEMES_CLIENT
        .fetch_themes(mal_id)
        .await
        .map_err(|e| e.to_string())
}
