import time
from datetime import datetime, timedelta
from PyQt6.QtCore import QObject, pyqtSignal
from utils.backend_bridge import BackendBridge

class ScheduleViewModel(QObject):
    state_changed = pyqtSignal(dict)  # {"loading": bool, "day_offset": int, "entries": list, "preferences": dict, "error": str | None}

    def __init__(self, parent=None):
        super().__init__(parent)
        self._day_offset = 0
        self._cache = {}  # day_offset -> list of AiringEntry
        self._preferences = {}  # media_id -> bool
        self._loading = False
        self._error = None

        BackendBridge.instance().error_occurred.connect(self._on_error)

    def _on_error(self, err_msg: str):
        self._loading = False
        self._error = err_msg
        self._notify_state()

    def load_day(self, day_offset: int, force_reload: bool = False):
        self._day_offset = day_offset
        self._error = None

        if not force_reload and day_offset in self._cache:
            self._loading = False
            self._notify_state()
            return

        self._loading = True
        self._notify_state()

        # Compute start of day and end of day Unix timestamps for the target date
        now = datetime.now()
        target_date = now + timedelta(days=day_offset)
        start_of_day = target_date.replace(hour=0, minute=0, second=0, microsecond=0)
        end_of_day = target_date.replace(hour=23, minute=59, second=59, microsecond=999999)

        from_ts = int(start_of_day.timestamp())
        to_ts = int(end_of_day.timestamp())

        # Load notification preferences first or concurrently
        BackendBridge.instance().list_notification_preferences(self._on_preferences_loaded)

        def _on_schedule_loaded(success, entries, err):
            self._loading = False
            if success:
                # Sort entries by airing_at
                sorted_entries = sorted(entries, key=lambda x: getattr(x, 'airing_at', 0))
                self._cache[day_offset] = sorted_entries
            else:
                self._error = err or "Failed to load schedule"
            self._notify_state()

        BackendBridge.instance().fetch_airing_schedule(from_ts, to_ts, _on_schedule_loaded)

    def _on_preferences_loaded(self, success, prefs, err):
        if success and prefs is not None:
            self._preferences = {p.media_id: p.enabled for p in prefs}
            self._notify_state()

    def toggle_notification(self, entry):
        media_id = getattr(entry, 'media_id', None)
        if not media_id:
            return

        current_enabled = self._preferences.get(media_id, False)
        new_enabled = not current_enabled
        self._preferences[media_id] = new_enabled
        self._notify_state()

        media_title = getattr(entry, 'media_title', 'Anime')
        cover_image = getattr(entry, 'cover_image', None)

        BackendBridge.instance().save_notification_preference(
            media_id=media_id,
            enabled=new_enabled,
            media_title=media_title,
            cover_image=cover_image
        )

    def _notify_state(self):
        entries = self._cache.get(self._day_offset, [])
        self.state_changed.emit({
            "loading": self._loading,
            "day_offset": self._day_offset,
            "entries": entries,
            "preferences": self._preferences,
            "error": self._error
        })
