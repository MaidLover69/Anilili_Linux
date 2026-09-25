use serde_json::json;
use std::env;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;

const CLIENT_ID: &str = "1219665725700771891";

pub struct DiscordRpc {
    stream: Mutex<Option<UnixStream>>,
}

impl DiscordRpc {
    pub fn new() -> Self {
        Self {
            stream: Mutex::new(None),
        }
    }

    fn find_socket() -> Option<String> {
        let runtime_dir = env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));

        for i in 0..10 {
            let path = format!("{}/discord-ipc-{}", runtime_dir, i);
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        let temp_dir = env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
        for i in 0..10 {
            let path = format!("{}/discord-ipc-{}", temp_dir, i);
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }

        None
    }

    fn connect(&self) -> bool {
        let mut guard = match self.stream.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if guard.is_some() {
            return true;
        }

        let sock_path = match Self::find_socket() {
            Some(p) => p,
            None => return false,
        };

        let mut stream = match UnixStream::connect(sock_path) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Handshake packet (Opcode 0)
        let handshake_payload = json!({
            "v": 1,
            "client_id": CLIENT_ID
        })
        .to_string();

        let mut packet = Vec::new();
        packet.extend_from_slice(&(0u32).to_le_bytes()); // Opcode 0 = Handshake
        packet.extend_from_slice(&(handshake_payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(handshake_payload.as_bytes());

        if stream.write_all(&packet).is_err() {
            return false;
        }

        *guard = Some(stream);
        true
    }

    pub fn set_activity(
        &self,
        anime_title: &str,
        episode_number: f64,
        is_playing: bool,
    ) {
        if !self.connect() {
            return;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let state_text = if episode_number > 0.0 {
            format!("Episode {}", episode_number)
        } else {
            "Browsing Anime".to_string()
        };

        let small_text = if is_playing { "Playing" } else { "Paused" };

        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": {
                    "details": anime_title,
                    "state": state_text,
                    "timestamps": {
                        "start": now
                    },
                    "assets": {
                        "large_image": "anilili_icon",
                        "large_text": "Anilili Linux",
                        "small_image": if is_playing { "play" } else { "pause" },
                        "small_text": small_text
                    }
                }
            },
            "nonce": format!("{}", now)
        })
        .to_string();

        let mut packet = Vec::new();
        packet.extend_from_slice(&(1u32).to_le_bytes()); // Opcode 1 = Frame
        packet.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(payload.as_bytes());

        if let Ok(mut guard) = self.stream.lock() {
            if let Some(ref mut s) = *guard {
                if s.write_all(&packet).is_err() {
                    *guard = None; // Reset if broken pipe
                }
            }
        }
    }

    pub fn clear_activity(&self) {
        if let Ok(mut guard) = self.stream.lock() {
            if let Some(ref mut s) = *guard {
                let payload = json!({
                    "cmd": "SET_ACTIVITY",
                    "args": {
                        "pid": std::process::id(),
                        "activity": serde_json::Value::Null
                    },
                    "nonce": "clear"
                })
                .to_string();

                let mut packet = Vec::new();
                packet.extend_from_slice(&(1u32).to_le_bytes());
                packet.extend_from_slice(&(payload.len() as u32).to_le_bytes());
                packet.extend_from_slice(payload.as_bytes());

                let _ = s.write_all(&packet);
            }
        }
    }
}
