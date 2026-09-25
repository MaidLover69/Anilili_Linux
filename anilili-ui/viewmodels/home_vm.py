from PyQt6.QtCore import QObject, pyqtSignal
from utils.backend_bridge import BackendBridge

class HomeViewModel(QObject):
    state_changed = pyqtSignal(dict)  # { "loading": bool, "data": dict, "error": str }

    def __init__(self, parent=None):
        super().__init__(parent)
        self.bridge = BackendBridge.instance()
        self.bridge.home_data_loaded.connect(self._on_home_data)
        self.bridge.error_occurred.connect(self._on_error)

        self._state = {
            "loading": True,
            "data": {
                "trending": [],
                "popular": [],
                "top_rated": [],
                "newest": [],
                "continue_watching": []
            },
            "error": None
        }

    def load_home_data(self):
        self._state["loading"] = True
        self._state["error"] = None
        self.state_changed.emit(self._state)
        self.bridge.fetch_home_data()

    def load(self):
        self.load_home_data()

    def _on_home_data(self, data: dict):
        self._state["loading"] = False
        self._state["data"] = data
        self._state["error"] = None
        self.state_changed.emit(self._state)

    def _on_error(self, err: str):
        self._state["loading"] = False
        self._state["error"] = err
        self.state_changed.emit(self._state)
