use crate::discord_rpc::DiscordRpc;
use once_cell::sync::Lazy;
use std::process::Command;

static DISCORD_RPC: Lazy<DiscordRpc> = Lazy::new(DiscordRpc::new);

#[tauri::command]
pub async fn update_discord_presence(
    anime_title: String,
    episode_number: f64,
    is_playing: bool,
) -> Result<(), String> {
    DISCORD_RPC.set_activity(&anime_title, episode_number, is_playing);
    Ok(())
}

#[tauri::command]
pub async fn clear_discord_presence() -> Result<(), String> {
    DISCORD_RPC.clear_activity();
    Ok(())
}

#[tauri::command]
pub async fn launch_external_player(
    player: String,
    url: String,
    title: Option<String>,
    referer: Option<String>,
) -> Result<bool, String> {
    let player_bin = match player.to_lowercase().as_str() {
        "vlc" => "vlc",
        _ => "mpv",
    };

    let mut cmd = Command::new(player_bin);

    if player_bin == "mpv" {
        cmd.arg(&url);
        if let Some(ref t) = title {
            cmd.arg(format!("--title={}", t));
        }
        if let Some(ref r) = referer {
            cmd.arg(format!("--http-header-fields=Referer: {}", r));
        }
    } else if player_bin == "vlc" {
        cmd.arg(&url);
        if let Some(ref t) = title {
            cmd.arg(format!("--meta-title={}", t));
        }
        if let Some(ref r) = referer {
            cmd.arg(format!(":http-referrer={}", r));
        }
    }

    match cmd.spawn() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to spawn {}: {}", player_bin, e)),
    }
}
