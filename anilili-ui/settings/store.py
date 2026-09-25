import json
import os
from pathlib import Path

DEFAULTS = {
    "autoplay": True,
    "auto_skip_intro_outro": False,
    "prefer_dub": False,
    "subtitles_with_dub": False,
    "default_quality": "highest",
    "player_gestures": True,
    "caption_text_scale": 100,
    "caption_text_color": "white",
    "caption_background_color": "black",
    "caption_background_opacity": 60,
    "caption_bold_text": True,
    "caption_bottom_margin": 12,
    "caption_edge_style": "none",
    "persistent_caption_delays": "",
    "download_quality": "best",
    "download_destination": "app_only",
    "download_dir": "",
    "hide_adult_content": True,
    "blur_episode_thumbnails": False,
    "auto_sync_anilist": True,
    "sync_watchlist_to_anilist": True,
    "release_notifications": True,
    "episode_layout": "list",
    "sidebar_expanded": True,
    "menu_language": "system",
    "preferred_provider": "auto",
    "server_priority": "",
    "last_pipe_origin": "",
    "enable_adult_providers": False,
    "update_check_on_launch": True,
    "window_width": 1280,
    "window_height": 780,
    "window_maximized": False,
    "anilist_viewer_id": None,
    "mal_token_expires_at": None,
}

class SettingsStore:
    _instance = None

    @classmethod
    def instance(cls) -> "SettingsStore":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self):
        self._path = Path.home() / ".config" / "anilili" / "settings.json"
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._data = self._load()

    def _load(self) -> dict:
        if self._path.exists():
            try:
                with open(self._path, "r", encoding="utf-8") as f:
                    return json.load(f)
            except (json.JSONDecodeError, OSError):
                return {}
        return {}

    def _save(self):
        try:
            with open(self._path, "w", encoding="utf-8") as f:
                json.dump(self._data, f, indent=2)
        except OSError as e:
            print(f"[SettingsStore] Failed to save settings: {e}")

    def get(self, key: str, default=None):
        return self._data.get(key, DEFAULTS.get(key, default))

    def set(self, key: str, value):
        self._data[key] = value
        self._save()

    def reset_to_defaults(self):
        self._data = {}
        self._save()
