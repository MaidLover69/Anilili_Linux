from PyQt6.QtCore import QObject, pyqtSignal, QTimer
from utils.backend_bridge import BackendBridge

class SearchViewModel(QObject):
    state_changed = pyqtSignal(dict)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.bridge = BackendBridge.instance()
        self.bridge.search_results_loaded.connect(self._on_results)
        self.bridge.error_occurred.connect(self._on_error)

        self._debounce_timer = QTimer(self)
        self._debounce_timer.setSingleShot(True)
        self._debounce_timer.setInterval(350)  # 350ms debounce
        self._debounce_timer.timeout.connect(self._execute_search)

        self.query = ""
        self.genres = []
        self.format = None
        self.status = None
        self.sort = "POPULARITY_DESC"
        self.page = 1
        self.per_page = 20

        self._results = []

        self._state = {
            "loading": False,
            "results": [],
            "page": 1,
            "error": None
        }

    def set_query(self, query: str):
        self.query = query
        self.page = 1
        self._results = []
        self._debounce_timer.start()

    def set_filters(self, genres=None, format=None, status=None, sort=None):
        if genres is not None:
            self.genres = genres
        if format is not None:
            self.format = format
        if status is not None:
            self.status = status
        if sort is not None:
            self.sort = sort

        self.page = 1
        self._results = []
        self._execute_search()

    def set_genres(self, genres: list):
        self.set_filters(genres=genres)

    def set_format(self, format_code: str):
        self.set_filters(format=format_code)

    def set_sort(self, sort_code: str):
        self.set_filters(sort=sort_code)

    def load_next_page(self):
        if self._state["loading"]:
            return
        # Disable infinite scroll on default empty browse (cap at 25 items)
        if not self.query and not self.genres and not self.format:
            return
        # Cap total search results at 100 items (5 pages x 20)
        if len(self._results) >= 100:
            return
        self.page += 1
        self._execute_search()


    def _execute_search(self):
        self._state["loading"] = True
        self.state_changed.emit(self._state)

        self.bridge.search_anime(
            query=self.query,
            genres=self.genres,
            format=self.format,
            status=self.status,
            sort=self.sort,
            page=self.page,
            per_page=self.per_page
        )

    def _on_results(self, new_items: list):
        self._state["loading"] = False
        if self.page == 1:
            self._results = new_items
        else:
            self._results.extend(new_items)

        self._state["results"] = self._results
        self._state["page"] = self.page
        self._state["error"] = None
        self._state["is_capped"] = (len(self._results) >= 100) or (not self.query and not self.genres and not self.format)
        self.state_changed.emit(self._state)

    def _on_error(self, err: str):
        self._state["loading"] = False
        self._state["error"] = err
        self._state["is_capped"] = False
        self.state_changed.emit(self._state)

