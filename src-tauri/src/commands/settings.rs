use crate::db;
use serde_json::Value;
use std::fs;

fn get_settings_path() -> Result<std::path::PathBuf, String> {
    let config_dir = db::get_config_dir().map_err(|e| e.to_string())?;
    Ok(config_dir.join("settings.json"))
}

#[tauri::command]
pub async fn get_settings() -> Result<Value, String> {
    let path = get_settings_path()?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            return Ok(val);
        }
    }
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn save_settings(settings: Value) -> Result<(), String> {
    let path = get_settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reset_settings() -> Result<Value, String> {
    let path = get_settings_path()?;
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    Ok(serde_json::json!({}))
}
