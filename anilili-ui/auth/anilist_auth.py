import base64
import json
import time
import keyring
from PyQt6.QtCore import QObject, pyqtSignal, QUrl
from PyQt6.QtWidgets import QDialog, QVBoxLayout
from PyQt6.QtWebEngineWidgets import QWebEngineView

# Client ID for AniList OAuth (Note: User should register their own client at anilist.co/settings/developer)
ANILIST_CLIENT_ID = "45552"
AUTH_URL = f"https://anilist.co/api/v2/oauth/authorize?client_id={ANILIST_CLIENT_ID}&response_type=token"
REDIRECT_PREFIX = "http://localhost"

class OAuthDialog(QDialog):
    token_received = pyqtSignal(str)

    def __init__(self, auth_url: str, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Connect AniList Account")
        self.resize(600, 700)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        self.web_view = QWebEngineView(self)
        layout.addWidget(self.web_view)

        self.web_view.urlChanged.connect(self._on_url_changed)
        self.web_view.load(QUrl(auth_url))

    def _on_url_changed(self, url: QUrl):
        url_str = url.toString()
        if url_str.startswith(REDIRECT_PREFIX):
            # Parse access token strictly from URL fragment (implicit grant flow)
            fragment = url.fragment()
            if fragment and "access_token=" in fragment:
                params = dict(item.split("=") for item in fragment.split("&") if "=" in item)
                if "access_token" in params:
                    token = params["access_token"]
                    self.token_received.emit(token)
                    self.accept()
                    return


class AniListAuthManager(QObject):
    _instance = None
    auth_state_changed = pyqtSignal(bool)
    auth_cancelled = pyqtSignal()

    @classmethod
    def instance(cls) -> "AniListAuthManager":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self, parent=None):
        super().__init__(parent)
        self._current_dialog = None

    def start_auth(self, parent_widget=None) -> None:
        self._current_dialog = OAuthDialog(AUTH_URL, parent_widget)
        self._current_dialog.token_received.connect(self.store_token)
        self._current_dialog.rejected.connect(self._on_dialog_rejected)
        self._current_dialog.exec()

    def _on_dialog_rejected(self):
        self.auth_cancelled.emit()

    def store_token(self, token: str) -> None:
        keyring.set_password("anilili", "anilist_token", token)
        
        # Parse JWT to extract viewer_id ("sub") and expiration ("exp")
        try:
            segments = token.split(".")
            if len(segments) >= 2:
                payload_seg = segments[1]
                # Fix base64url padding
                padding = 4 - (len(payload_seg) % 4)
                if padding < 4:
                    payload_seg += "=" * padding
                decoded_bytes = base64.urlsafe_b64decode(payload_seg)
                payload_data = json.loads(decoded_bytes.decode("utf-8"))
                
                sub_id = payload_data.get("sub")
                if sub_id:
                    from settings.store import SettingsStore
                    SettingsStore.instance().set("anilist_viewer_id", int(sub_id))
        except Exception as e:
            print(f"[AniListAuthManager] Error parsing JWT payload: {e}")

        self.auth_state_changed.emit(True)

    def get_token(self) -> str | None:
        try:
            return keyring.get_password("anilili", "anilist_token")
        except Exception:
            return None

    def is_authenticated(self) -> bool:
        token = self.get_token()
        if not token:
            return False

        # Verify JWT exp claim if present
        try:
            segments = token.split(".")
            if len(segments) >= 2:
                payload_seg = segments[1]
                padding = 4 - (len(payload_seg) % 4)
                if padding < 4:
                    payload_seg += "=" * padding
                decoded_bytes = base64.urlsafe_b64decode(payload_seg)
                payload_data = json.loads(decoded_bytes.decode("utf-8"))

                exp = payload_data.get("exp")
                if exp and time.time() >= float(exp):
                    return False
        except Exception:
            pass

        return True

    def logout(self) -> None:
        try:
            keyring.delete_password("anilili", "anilist_token")
        except Exception:
            pass

        from settings.store import SettingsStore
        SettingsStore.instance().set("anilist_viewer_id", None)
        self.auth_state_changed.emit(False)
