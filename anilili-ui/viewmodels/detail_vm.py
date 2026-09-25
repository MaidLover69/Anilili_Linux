import asyncio
from PyQt6.QtCore import QObject, pyqtSignal
from utils.backend_bridge import BackendBridge

class InstantEpisodeItem:
    def __init__(self, number: float, title: str | None = None, image: str | None = None, filler: bool = False):
        self.number = number
        self.pipe_id = f"anilist:{number}"
        self.title = title or f"Episode {int(number) if number.is_integer() else number}"
        self.image = image
        self.filler = filler

def generate_instant_catalog(details: dict) -> list:
    ep_count = 1
    if details:
        next_airing = details.get("nextAiringEpisode")
        if next_airing and isinstance(next_airing, dict) and next_airing.get("episode"):
            ep_count = max(1, next_airing["episode"] - 1)
        elif details.get("episodes"):
            ep_count = max(1, details["episodes"])

    return [InstantEpisodeItem(float(i)) for i in range(1, ep_count + 1)]


class DetailViewModel(QObject):
    state_changed = pyqtSignal(dict)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.bridge = BackendBridge.instance()

        self.bridge.anime_details_loaded.connect(self._on_details)
        self.bridge.episodes_loaded.connect(self._on_episodes)
        self.bridge.konoha_data_loaded.connect(self._on_konoha)
        self.bridge.watched_episodes_loaded.connect(self._on_watched)
        self.bridge.error_occurred.connect(self._on_error)

        self._state = {
            "loading": True,
            "details": None,
            "episodes": {},
            "konoha": [],
            "watched": [],
            "error": None
        }

    def load(self, anilist_id: int, mal_id: int | None = None):
        self._state["loading"] = True
        self._state["error"] = None
        self._state["details"] = None
        self._state["episodes"] = {}
        self._state["konoha"] = []
        self._state["watched"] = []
        self.state_changed.emit(self._state)

        # Execute parallel async calls via BackendBridge (instant metadata + konoha + watched)
        self.bridge.load_anime_details(anilist_id)
        self.bridge.load_konoha_data(anilist_id)
        self.bridge.load_watched_episodes(anilist_id)

    def _on_details(self, details: dict):
        self._state["details"] = details
        # Generate instant count-based catalog immediately
        instant_eps = generate_instant_catalog(details)
        self._state["episodes"] = {
            "AniList": {
                "sub": instant_eps,
                "dub": instant_eps.copy()
            }
        }
        self._check_loading_complete()

        # Extract mal_id & title_romaji from details dict
        anilist_id = details.get("id")
        mal_id = details.get("idMal")
        title_obj = details.get("title", {}) if isinstance(details.get("title"), dict) else {}
        title_romaji = title_obj.get("romaji") or title_obj.get("userPreferred") or title_obj.get("english") or ""

        print(f"[DetailVM] _on_details: trigger load_episodes(anilist_id={anilist_id}, mal_id={mal_id}, title_romaji='{title_romaji}')")

        # Trigger real provider episode catalog fetch
        if anilist_id:
            self.bridge.load_episodes(anilist_id, mal_id, title_romaji)

    def _on_episodes(self, episodes: dict):
        print(f"[DetailVM] _on_episodes received providers: {list(episodes.keys()) if isinstance(episodes, dict) else episodes}")
        if episodes:
            self._state["episodes"] = episodes
            self.state_changed.emit(self._state)

    def _on_konoha(self, konoha: list):
        self._state["konoha"] = konoha
        # Asynchronously enrich existing catalog items with Konoha titles & images
        if konoha and self._state["episodes"]:
            konoha_map = {round(item.number): item for item in konoha if hasattr(item, 'number')}
            for p_name, p_data in self._state["episodes"].items():
                for cat in ["sub", "dub"]:
                    for item in p_data.get(cat, []):
                        ep_num = round(item.number)
                        if ep_num in konoha_map:
                            k_item = konoha_map[ep_num]
                            if hasattr(k_item, 'title') and k_item.title:
                                item.title = k_item.title
                            if hasattr(k_item, 'image') and k_item.image:
                                item.image = k_item.image
                            if hasattr(k_item, 'filler'):
                                item.filler = k_item.filler
        self._check_loading_complete()

    def _on_watched(self, watched: list):
        self._state["watched"] = watched
        self._check_loading_complete()

    def _check_loading_complete(self):
        if self._state["details"] is not None:
            self._state["loading"] = False
            self.state_changed.emit(self._state)

    def _on_error(self, err: str):
        if self._state["details"] is None:
            self._state["loading"] = False
            self._state["error"] = err
            self.state_changed.emit(self._state)

    def get_next_unwatched_episode(self, episode_list: list, watched_list: list) -> float:
        if not episode_list:
            return 1.0

        sorted_eps = sorted(episode_list, key=lambda x: x.number)
        for ep in sorted_eps:
            is_w = any(abs(ep.number - w) < 0.01 for w in watched_list)
            if not is_w:
                return ep.number

        return sorted_eps[0].number

    def toggle_watched(self, episode_number: float, is_watched: bool):
        """Toggle the watched state for a given episode number."""
        watched = self._state["watched"]
        already_in = any(abs(episode_number - w) < 0.01 for w in watched)

        if is_watched and not already_in:
            watched.append(episode_number)
        elif not is_watched and already_in:
            self._state["watched"] = [w for w in watched if abs(w - episode_number) >= 0.01]

        # Persist via BackendBridge — save a "completed" watch progress entry
        details = self._state.get("details") or {}
        anilist_id = details.get("id")
        title_obj = details.get("title", {}) if isinstance(details.get("title"), dict) else {}
        title = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or ""

        if anilist_id and is_watched:
            self.bridge.save_watch_progress(
                anilist_id=anilist_id,
                title=title,
                cover=None,
                episode_number=episode_number,
                episode_title=f"Episode {int(episode_number) if episode_number == int(episode_number) else episode_number}",
                provider="manual",
                category="sub",
                position_ms=0,
                duration_ms=0
            )

        self.state_changed.emit(self._state)
