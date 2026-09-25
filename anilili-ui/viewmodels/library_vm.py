import os
import gzip
import defusedxml.ElementTree as ET
from PyQt6.QtCore import QObject, pyqtSignal
from utils.backend_bridge import BackendBridge
from auth import AniListAuthManager, MalAuthManager
from settings.store import SettingsStore

# Tab display label to AniList GraphQL status string mapping
TAB_STATUS_MAP = {
    "Watching": "CURRENT",
    "Planning": "PLANNING",
    "Completed": "COMPLETED",
    "Paused": "PAUSED",
    "Dropped": "DROPPED",
    "Re-watching": "REPEATING",
}

MAL_STATUS_MAP = {
    "Watching": "CURRENT",
    "Completed": "COMPLETED",
    "On-Hold": "PAUSED",
    "Dropped": "DROPPED",
    "Plan to Watch": "PLANNING",
}

class LibraryViewModel(QObject):
    library_loaded = pyqtSignal(list)
    viewer_loaded = pyqtSignal(object)
    error = pyqtSignal(str)
    loading = pyqtSignal(bool)
    toast_message = pyqtSignal(str)
    reload_requested = pyqtSignal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.bridge = BackendBridge.instance()
        self._current_status = "CURRENT"
        self._all_entries = []  # Stores full loaded entries for instant in-memory filtering

        self.bridge.anilist_library_loaded.connect(self._on_library_loaded)
        self.bridge.anilist_viewer_loaded.connect(self._on_viewer_loaded)
        self.bridge.error_occurred.connect(self._on_error)
        self.bridge.library_entries_loaded.connect(self._on_library_loaded)

    def load(self, tab_label: str = "Watching"):
        status = TAB_STATUS_MAP.get(tab_label, "CURRENT")
        self._current_status = status
        self.loading.emit(True)

        auth_mgr = AniListAuthManager.instance()
        if not auth_mgr.is_authenticated():
            # Logged-out state or fallback to local db
            self.bridge.load_library_entries(status)
            return

        token = auth_mgr.get_token()
        viewer_id = SettingsStore.instance().get("anilist_viewer_id")

        if viewer_id is None and token:
            # Fallback: fetch viewer first to obtain viewer_id before loading library
            self.bridge.load_anilist_viewer(token)
        else:
            self.bridge.load_anilist_viewer(token)
            self.bridge.load_anilist_library(token, user_id=int(viewer_id), status=status)

    def switch_tab(self, tab_label: str):
        self._all_entries = []
        self.load(tab_label)

    def filter_entries(self, query: str):
        if not query or not query.strip():
            self.library_loaded.emit(self._all_entries)
            return

        q = query.strip().lower()
        filtered = [
            e for e in self._all_entries
            if e.title and q in e.title.lower()
        ]
        self.library_loaded.emit(filtered)

    def logout_anilist(self):
        AniListAuthManager.instance().logout()
        self._all_entries = []
        self.reload_requested.emit()

    def logout_mal(self):
        MalAuthManager.instance().logout()
        self.reload_requested.emit()

    def import_mal_xml(self, file_path: str):
        if not file_path or not os.path.exists(file_path):
            self.error.emit("Selected file does not exist.")
            return

        try:
            # Handle .xml.gz vs raw .xml
            if file_path.endswith(".gz"):
                with gzip.open(file_path, "rb") as f:
                    tree = ET.parse(f)
            else:
                tree = ET.parse(file_path)

            root = tree.getroot()
            count = 0

            for anime in root.findall("anime"):
                mal_id_str = anime.findtext("series_animedb_id")
                if not mal_id_str:
                    continue
                mal_id = int(mal_id_str)
                title = anime.findtext("series_title") or f"MAL Anime {mal_id}"
                mal_status = anime.findtext("my_status") or "Plan to Watch"
                progress = int(anime.findtext("my_watched_episodes") or 0)
                score = float(anime.findtext("my_score") or 0.0)

                anilist_status = MAL_STATUS_MAP.get(mal_status, "PLANNING")

                # Upsert entry into local remote_library table
                self.bridge.upsert_library_entry(
                    anilist_id=mal_id,  # proxy ID
                    status=anilist_status,
                    progress=progress,
                    score=score,
                    title=title,
                    cover=None,
                    format_str="TV",
                    total_episodes=None,
                    average_score=int(score * 10) if score > 0 else None
                )
                count += 1

            self.toast_message.emit(f"Imported {count} anime from MAL")
            self.reload_requested.emit()

        except Exception as ex:
            self.error.emit(f"Failed to parse MAL XML: {ex}")

    def _on_viewer_loaded(self, viewer):
        if viewer:
            from settings.store import SettingsStore
            SettingsStore.instance().set("anilist_viewer_id", viewer.id)
            self.viewer_loaded.emit(viewer)

            # If library load was pending on viewer_id
            token = AniListAuthManager.instance().get_token()
            if token and viewer.id:
                self.bridge.load_anilist_library(token, user_id=viewer.id, status=self._current_status)

    def _on_library_loaded(self, entries):
        self._all_entries = entries
        self.loading.emit(False)
        self.library_loaded.emit(entries)

    def _on_error(self, err_msg: str):
        self.loading.emit(False)
        self.error.emit(err_msg)
