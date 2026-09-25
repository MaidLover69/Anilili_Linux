import base64
import secrets
import time
import httpx
import keyring
from PyQt6.QtCore import QObject, pyqtSignal, QUrl
from PyQt6.QtWidgets import QDialog, QVBoxLayout
from PyQt6.QtWebEngineWidgets import QWebEngineView
from settings.store import SettingsStore

# MAL Client ID (Note: User should register their app at myanimelist.net/apiconfig)
MAL_CLIENT_ID = "4ae5f8056db821737a4f40f8816177cd"
MAL_TOKEN_URL = "https://myanimelist.net/v1/oauth2/token"
MAL_REDIRECT_PREFIX = "https://myanimelist.net/oauth2/callback"

class MalOAuthDialog(QDialog):
    code_received = pyqtSignal(str)

    def __init__(self, auth_url: str, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Connect MyAnimeList Account")
        self.resize(600, 700)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        self.web_view = QWebEngineView(self)
        layout.addWidget(self.web_view)

        self.web_view.urlChanged.connect(self._on_url_changed)
        self.web_view.load(QUrl(auth_url))

    def _on_url_changed(self, url: QUrl):
        url_str = url.toString()
        if url_str.startswith(MAL_REDIRECT_PREFIX):
            # Extract authorization code from query parameter
            query = url.query()
            if query and "code=" in query:
                params = dict(item.split("=") for item in query.split("&") if "=" in item)
                if "code" in params:
                    code = params["code"]
                    self.code_received.emit(code)
                    self.accept()
                    return


class MalAuthManager(QObject):
    _instance = None
    auth_state_changed = pyqtSignal(bool)
    auth_cancelled = pyqtSignal()

    @classmethod
    def instance(cls) -> "MalAuthManager":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self, parent=None):
        super().__init__(parent)
        self._verifier = None
        self._current_dialog = None

    def start_auth(self, parent_widget=None) -> None:
        # Generate PKCE verifier (plain method: challenge == verifier)
        raw_bytes = secrets.token_bytes(32)
        verifier = base64.urlsafe_b64encode(raw_bytes).rstrip(b"=").decode("utf-8")
        challenge = verifier
        self._verifier = verifier

        auth_url = (
            "https://myanimelist.net/v1/oauth2/authorize"
            f"?client_id={MAL_CLIENT_ID}&response_type=code"
            f"&code_challenge={challenge}&code_challenge_method=plain"
        )

        self._current_dialog = MalOAuthDialog(auth_url, parent_widget)
        self._current_dialog.code_received.connect(self._exchange_code)
        self._current_dialog.rejected.connect(self._on_dialog_rejected)
        self._current_dialog.exec()

    def _on_dialog_rejected(self):
        self.auth_cancelled.emit()

    def _exchange_code(self, code: str) -> None:
        if not self._verifier:
            self.auth_cancelled.emit()
            return

        data = {
            "client_id": MAL_CLIENT_ID,
            "grant_type": "authorization_code",
            "code": code,
            "code_verifier": self._verifier,
        }

        try:
            with httpx.Client() as client:
                res = client.post(MAL_TOKEN_URL, data=data)
                if res.status_code == 200:
                    resp_json = res.json()
                    access_token = resp_json.get("access_token")
                    refresh_token = resp_json.get("refresh_token")
                    expires_in = resp_json.get("expires_in", 2592000)

                    if access_token:
                        keyring.set_password("anilili", "mal_access_token", access_token)
                        if refresh_token:
                            keyring.set_password("anilili", "mal_refresh_token", refresh_token)
                        SettingsStore.instance().set("mal_token_expires_at", time.time() + expires_in)
                        self.auth_state_changed.emit(True)
                        return
        except Exception as e:
            print(f"[MalAuthManager] Token exchange error: {e}")

        self.auth_cancelled.emit()

    def get_access_token(self) -> str | None:
        try:
            access_token = keyring.get_password("anilili", "mal_access_token")
            if not access_token:
                return None

            expires_at = SettingsStore.instance().get("mal_token_expires_at")
            if expires_at and (expires_at - time.time() < 300):
                if self.refresh_token():
                    access_token = keyring.get_password("anilili", "mal_access_token")

            return access_token
        except Exception:
            return None

    def refresh_token(self) -> bool:
        try:
            refresh_token = keyring.get_password("anilili", "mal_refresh_token")
            if not refresh_token:
                return False

            data = {
                "client_id": MAL_CLIENT_ID,
                "grant_type": "refresh_token",
                "refresh_token": refresh_token,
            }

            with httpx.Client() as client:
                res = client.post(MAL_TOKEN_URL, data=data)
                if res.status_code == 200:
                    resp_json = res.json()
                    new_access = resp_json.get("access_token")
                    new_refresh = resp_json.get("refresh_token")
                    expires_in = resp_json.get("expires_in", 2592000)

                    if new_access:
                        keyring.set_password("anilili", "mal_access_token", new_access)
                        if new_refresh:
                            keyring.set_password("anilili", "mal_refresh_token", new_refresh)
                        SettingsStore.instance().set("mal_token_expires_at", time.time() + expires_in)
                        return True
        except Exception as e:
            print(f"[MalAuthManager] Token refresh error: {e}")

        return False

    def is_authenticated(self) -> bool:
        return self.get_access_token() is not None

    def logout(self) -> None:
        try:
            keyring.delete_password("anilili", "mal_access_token")
        except Exception:
            pass
        try:
            keyring.delete_password("anilili", "mal_refresh_token")
        except Exception:
            pass

        SettingsStore.instance().set("mal_token_expires_at", None)
        self.auth_state_changed.emit(False)
