import json
import anilili_core
from PyQt6.QtCore import QObject, pyqtSignal

class BackendBridge(QObject):
    _instance = None

    # Home & Discover signals
    home_data_loaded = pyqtSignal(dict)
    search_results_loaded = pyqtSignal(list)
    trending_loaded = pyqtSignal(list)
    popular_loaded = pyqtSignal(list)
    top_rated_loaded = pyqtSignal(list)
    continue_watching_loaded = pyqtSignal(list)
    error_occurred = pyqtSignal(str)

    # Detail signals
    anime_details_loaded = pyqtSignal(dict)
    episodes_loaded = pyqtSignal(dict)
    konoha_data_loaded = pyqtSignal(list)
    watched_episodes_loaded = pyqtSignal(list)

    # Watch & Player signals
    episode_sources_loaded = pyqtSignal(list)
    skip_times_loaded = pyqtSignal(dict)
    watch_progress_saved = pyqtSignal()
    episode_progress_loaded = pyqtSignal(int, int)  # position_ms, duration_ms

    # Phase 4 Auth & Library & Sync signals
    anilist_viewer_loaded = pyqtSignal(object)
    anilist_library_loaded = pyqtSignal(list)
    anilist_entry_saved = pyqtSignal(object)
    mal_status_saved = pyqtSignal()
    watchlist_loaded = pyqtSignal(list)
    watchlist_modified = pyqtSignal()
    library_entries_loaded = pyqtSignal(list)

    # Phase 5 Download signals
    downloads_loaded = pyqtSignal(list)
    download_record_updated = pyqtSignal(object)

    # Phase 6 Schedule & Notification signals
    airing_schedule_loaded = pyqtSignal(list)
    notification_pref_loaded = pyqtSignal(object)
    notification_pref_saved = pyqtSignal()
    filler_episodes_loaded = pyqtSignal(list)

    # Phase 7 Update signals
    update_available = pyqtSignal(object)
    no_update = pyqtSignal()



    @classmethod
    def instance(cls) -> "BackendBridge":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self, parent=None):
        super().__init__(parent)
        # Synchronous in-memory cache for never-regress check
        self._library_cache = {}  # media_id -> {"progress": int, "status": str}
        # Deduplication set for rapid/concurrent sync calls
        self._pending_syncs = set()  # set of (media_id, progress)

    def fetch_home_data(self):
        def _cb(success, data, err):
            if success:
                self.home_data_loaded.emit(data)
            else:
                self.error_occurred.emit(err or "Failed to load home data")
        anilili_core.fetch_home_data(_cb)

    def fetch_trending(self):
        def _cb(success, media_list, err):
            if success:
                self.trending_loaded.emit(media_list)
            else:
                self.error_occurred.emit(err or "Failed to load trending anime")
        anilili_core.fetch_trending(_cb)

    def fetch_popular(self):
        def _cb(success, media_list, err):
            if success:
                self.popular_loaded.emit(media_list)
            else:
                self.error_occurred.emit(err or "Failed to load popular anime")
        anilili_core.fetch_popular(_cb)

    def fetch_top_rated(self):
        def _cb(success, media_list, err):
            if success:
                self.top_rated_loaded.emit(media_list)
            else:
                self.error_occurred.emit(err or "Failed to load top-rated anime")
        anilili_core.fetch_top_rated(_cb)

    def search_anime(self, query=None, genres=None, format=None, status=None, sort=None, page=1, per_page=20):
        if genres is None:
            genres = []
        def _cb(success, media_list, err):
            if success:
                self.search_results_loaded.emit(media_list)
            else:
                self.error_occurred.emit(err or "Failed to execute search")
        anilili_core.search_anime(
            query=query,
            genres=genres,
            format=format,
            status=status,
            sort=sort,
            page=page,
            per_page=per_page,
            callback=_cb
        )

    def get_continue_watching(self):
        def _cb(success, entries, err):
            if success:
                self.continue_watching_loaded.emit(entries)
            else:
                self.error_occurred.emit(err or "Failed to load watch history")
        anilili_core.get_continue_watching(_cb)

    def load_anime_details(self, anilist_id: int):
        def _cb(success, data_str, err):
            if success:
                try:
                    data = json.loads(data_str)
                    self.anime_details_loaded.emit(data)
                except Exception as ex:
                    self.error_occurred.emit(f"Parse error: {ex}")
            else:
                self.error_occurred.emit(err or "Failed to load anime details")
        anilili_core.fetch_anime_details(anilist_id, _cb)

    def load_episodes(self, anilist_id: int, mal_id: int | None = None, title_romaji: str | None = None):
        def _cb(success, provider_dict, err):
            if success:
                self.episodes_loaded.emit(provider_dict)
            else:
                self.error_occurred.emit(err or "Failed to load provider episodes")
        anilili_core.get_all_episodes(anilist_id, mal_id, title_romaji, _cb)

    def load_konoha_data(self, anilist_id: int):
        def _cb(success, episodes, err):
            if success:
                self.konoha_data_loaded.emit(episodes)
            else:
                self.konoha_data_loaded.emit([])
        anilili_core.get_konoha_episodes(anilist_id, _cb)

    def load_watched_episodes(self, anilist_id: int):
        def _cb(success, episodes, err):
            if success:
                self.watched_episodes_loaded.emit(episodes)
            else:
                self.watched_episodes_loaded.emit([])
        anilili_core.get_watched_episodes(anilist_id, _cb)

    def load_episode_sources(self, anilist_id: int, mal_id: int | None = None, episode_number: float = 1.0, category: str = "sub", title_romaji: str | None = None):
        def _cb(success, streams, err):
            if success:
                self.episode_sources_loaded.emit(streams)
            else:
                self.error_occurred.emit(err or "Failed to load episode stream sources")
        anilili_core.get_episode_sources(anilist_id, mal_id, episode_number, category, title_romaji, _cb)

    def load_skip_times(self, mal_id: int | None, episode_number: float, duration_s: float):
        if not mal_id:
            self.skip_times_loaded.emit({})
            return
        def _cb(success, dict_data, err):
            if success:
                self.skip_times_loaded.emit(dict_data)
            else:
                self.skip_times_loaded.emit({})
        anilili_core.get_skip_times(mal_id, episode_number, duration_s, _cb)

    def load_episode_progress(self, anilist_id: int, episode_number: float):
        def _cb(success, pos, dur, err):
            if success:
                self.episode_progress_loaded.emit(pos, dur)
            else:
                self.episode_progress_loaded.emit(0, 0)
        anilili_core.get_episode_progress(anilist_id, episode_number, _cb)

    def save_watch_progress(self, anilist_id: int, title: str, cover: str | None, episode_number: float, episode_title: str | None, provider: str, category: str, position_ms: int, duration_ms: int):
        def _cb(success, err):
            if success:
                self.watch_progress_saved.emit()
            else:
                self.error_occurred.emit(err or "Failed to save watch progress")
        anilili_core.update_watch_progress(
            anilist_id=anilist_id,
            title=title,
            cover=cover,
            episode_number=episode_number,
            episode_title=episode_title,
            provider=provider,
            category=category,
            position_ms=position_ms,
            duration_ms=duration_ms,
            callback=_cb
        )

    # --- Phase 4 Auth & Sync Bridge Methods ---

    def load_anilist_viewer(self, token: str):
        def _cb(success, viewer, err):
            if success:
                self.anilist_viewer_loaded.emit(viewer)
            else:
                self.error_occurred.emit(err or "Failed to load AniList viewer info")
        anilili_core.get_anilist_viewer(token, _cb)

    def load_anilist_library(self, token: str, user_id: i32, status: str | None = None):
        def _cb(success, entries, err):
            if success:
                # Pre-populate synchronous _library_cache and upsert into local DB
                for entry in entries:
                    self._library_cache[entry.id] = {
                        "progress": entry.progress,
                        "status": entry.status,
                    }
                    anilili_core.upsert_library_entry(
                        anilist_id=entry.id,
                        status=entry.status,
                        progress=entry.progress,
                        score=entry.score,
                        title=entry.title,
                        cover=entry.cover,
                        format_str=entry.format_str,
                        total_episodes=entry.total_episodes,
                        average_score=entry.average_score
                    )
                self.anilist_library_loaded.emit(entries)
            else:
                self.error_occurred.emit(err or "Failed to load AniList library")
        anilili_core.fetch_anilist_library(token, user_id, status, _cb)

    def save_anilist_progress(self, token: str, media_id: int, progress: int, status: str | None = "CURRENT"):
        sync_pair = (media_id, progress)

        # 1. Deduplication check: if (media_id, progress) is currently pending, skip
        if sync_pair in self._pending_syncs:
            return

        # 2. Synchronous never-regress check using in-memory _library_cache
        cached = self._library_cache.get(media_id)
        if cached:
            existing_progress = cached.get("progress", 0)
            existing_status = cached.get("status")
            if existing_progress >= progress or existing_status == "COMPLETED":
                # Never regress progress or mutate completed entries
                return

        # Mark as pending
        self._pending_syncs.add(sync_pair)

        def _cb(success, entry, err):
            self._pending_syncs.discard(sync_pair)
            if success and entry:
                # Update in-memory cache and emit signal
                self._library_cache[media_id] = {
                    "progress": entry.progress,
                    "status": entry.status,
                }
                self.anilist_entry_saved.emit(entry)

        anilili_core.save_anilist_entry(
            token=token,
            media_id=media_id,
            status=status,
            progress=progress,
            score=None,
            callback=_cb
        )

    def save_mal_progress(self, access_token: str, mal_id: int, episode: int, status: str | None = "CURRENT"):
        def _cb(success, err):
            if success:
                self.mal_status_saved.emit()
            else:
                # Non-blocking sync error: log warning
                print(f"[BackendBridge] MAL sync error for ID {mal_id}: {err}")
        anilili_core.mal_update_status(access_token, mal_id, episode, status, _cb)

    def add_to_watchlist(self, anilist_id: int, title: str, cover: str | None = None, format: str | None = None, average_score: int | None = None):
        def _cb(success, err):
            if success:
                self.watchlist_modified.emit()
            else:
                self.error_occurred.emit(err or "Failed to add to watchlist")
        anilili_core.add_to_watchlist(anilist_id, title, cover, format, average_score, _cb)

    def remove_from_watchlist(self, anilist_id: int):
        def _cb(success, err):
            if success:
                self.watchlist_modified.emit()
            else:
                self.error_occurred.emit(err or "Failed to remove from watchlist")
        anilili_core.remove_from_watchlist(anilist_id, _cb)

    def is_in_watchlist(self, anilist_id: int, callback=None):
        def _cb(success, is_in, err):
            if callback:
                callback(success, is_in, err)
        anilili_core.is_in_watchlist(anilist_id, _cb)


    def load_watchlist(self):
        def _cb(success, entries, err):
            if success:
                self.watchlist_loaded.emit(entries)
            else:
                self.error_occurred.emit(err or "Failed to load watchlist")
        anilili_core.get_watchlist(_cb)

    def load_library_entries(self, status_filter: str | None = None):
        def _cb(success, entries, err):
            if success:
                for entry in entries:
                    self._library_cache[entry.id] = {
                        "progress": entry.progress,
                        "status": entry.status,
                    }
                self.library_entries_loaded.emit(entries)
            else:
                self.error_occurred.emit(err or "Failed to load library entries")
        anilili_core.get_library_entries(status_filter, _cb)

    # --- Phase 5 Download Methods ---

    def create_download(self, record):
        def _cb(success, id_str, err):
            if not success:
                self.error_occurred.emit(err or "Failed to create download record")
        anilili_core.create_download(record, _cb)

    def update_download_status(self, download_id: str, status: str, progress: float | None = None, file_path: str | None = None, file_size: int | None = None, duration_s: float | None = None, error_msg: str | None = None):
        def _cb(success, err):
            if not success:
                print(f"[BackendBridge] update_download_status failed: {err}")
        anilili_core.update_download_status(download_id, status, progress, file_path, file_size, duration_s, error_msg, _cb)

    def load_downloads(self, anilist_id: int | None = None):
        def _cb(success, records, err):
            if success:
                self.downloads_loaded.emit(records)
            else:
                self.error_occurred.emit(err or "Failed to load downloads")

        if anilist_id:
            anilili_core.get_downloads(anilist_id, _cb)
        else:
            anilili_core.get_all_downloads(_cb)

    def delete_download(self, download_id: str):
        def _cb(success, err):
            if not success:
                self.error_occurred.emit(err or "Failed to delete download")
        anilili_core.delete_download(download_id, _cb)

    def get_download_by_id(self, download_id: str, callback=None):
        def _cb(success, rec, err):
            if callback:
                callback(success, rec, err)
        anilili_core.get_download_by_id(download_id, _cb)

    # --- Phase 6 Airing Schedule & Notification Methods ---

    def fetch_airing_schedule(self, from_ts: int, to_ts: int, callback=None):
        def _cb(success, entries, err):
            if success:
                self.airing_schedule_loaded.emit(entries)
            else:
                self.error_occurred.emit(err or "Failed to load airing schedule")
            if callback:
                callback(success, entries, err)
        anilili_core.fetch_airing_schedule(from_ts, to_ts, _cb)

    def get_filler_episodes(self, mal_id: int, callback=None):
        def _cb(success, episodes, err):
            if success:
                self.filler_episodes_loaded.emit(episodes)
            if callback:
                callback(success, episodes, err)
        anilili_core.get_filler_episodes(mal_id, _cb)

    def save_notification_preference(self, media_id: int, enabled: bool, media_title: str, cover_image: str | None = None, callback=None):
        pref = anilili_core.NotificationPreference(
            media_id=media_id,
            enabled=enabled,
            media_title=media_title,
            cover_image=cover_image
        )
        def _cb(success, err):
            if success:
                self.notification_pref_saved.emit()
            else:
                self.error_occurred.emit(err or "Failed to save notification preference")
            if callback:
                callback(success, err)
        anilili_core.save_notification_preference(pref, _cb)

    def get_notification_preference(self, media_id: int, callback=None):
        def _cb(success, pref, err):
            if success:
                self.notification_pref_loaded.emit(pref)
            if callback:
                callback(success, pref, err)
        anilili_core.get_notification_preference(media_id, _cb)

    def list_notification_preferences(self, callback=None):
        def _cb(success, prefs, err):
            if callback:
                callback(success, prefs, err)
        anilili_core.list_notification_preferences(_cb)

    def check_for_update(self, callback=None):
        def _cb(success, info, err):
            if callback:
                callback(success, info, err)
            if success and info:
                self.update_available.emit(info)
            else:
                self.no_update.emit()
        anilili_core.check_for_update(_cb)



