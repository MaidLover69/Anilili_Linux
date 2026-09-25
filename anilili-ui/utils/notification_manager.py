import time
from PyQt6.QtCore import QObject, QTimer
from utils.backend_bridge import BackendBridge

try:
    import notify2
    HAS_NOTIFY2 = True
except ImportError:
    HAS_NOTIFY2 = False

class NotificationManager(QObject):
    _instance = None

    @classmethod
    def instance(cls) -> "NotificationManager":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self, parent=None):
        super().__init__(parent)
        self._notified_episodes = set()  # set of (media_id, episode)
        self._is_initialized = False
        self._check_timer = QTimer(self)
        self._check_timer.setInterval(60000)  # Check every 60 seconds
        self._check_timer.timeout.connect(self._check_airing_notifications)

    def start(self):
        if HAS_NOTIFY2 and not self._is_initialized:
            try:
                notify2.init("Anilili")
                self._is_initialized = True
            except Exception as e:
                print(f"[NotificationManager] notify2 init warning: {e}")

        if not self._check_timer.isActive():
            self._check_timer.start()

    def stop(self):
        if self._check_timer.isActive():
            self._check_timer.stop()

    def _check_airing_notifications(self):
        BackendBridge.instance().list_notification_preferences(self._on_preferences_loaded)

    def _on_preferences_loaded(self, success, prefs, err):
        if not success or not prefs:
            return

        enabled_prefs = {p.media_id: p for p in prefs if getattr(p, 'enabled', True)}
        if not enabled_prefs:
            return

        now_ts = int(time.time())
        from_ts = now_ts - 300
        to_ts = now_ts + 300

        def _on_schedule_loaded(sched_success, entries, sched_err):
            if not sched_success or not entries:
                return

            for entry in entries:
                media_id = getattr(entry, 'media_id', None)
                if media_id in enabled_prefs:
                    ep_num = getattr(entry, 'episode', 1)
                    key = (media_id, ep_num)
                    if key not in self._notified_episodes:
                        airing_at = getattr(entry, 'airing_at', 0)
                        if abs(now_ts - airing_at) <= 300:
                            self._send_notification(entry)
                            self._notified_episodes.add(key)

        BackendBridge.instance().fetch_airing_schedule(from_ts, to_ts, _on_schedule_loaded)

    def _send_notification(self, entry):
        title = f"Anilili — {entry.media_title}"
        message = f"Episode {entry.episode} is now airing!"
        print(f"[NotificationManager] Firing desktop notification: {title} - {message}")

        if HAS_NOTIFY2 and self._is_initialized:
            try:
                n = notify2.Notification(title, message, icon="video-display")
                n.set_timeout(10000)  # 10 seconds display
                n.show()
            except Exception as e:
                print(f"[NotificationManager] Notification show error: {e}")
