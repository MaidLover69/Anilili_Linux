from PyQt6.QtCore import pyqtSignal, Qt, QTimer
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLineEdit, QComboBox,
    QFrame, QPushButton, QGridLayout, QLabel, QScrollArea
)
from components.anime_card import AnimeCard
from viewmodels.search_vm import SearchViewModel


GENRES = [
    "Action", "Adventure", "Comedy", "Drama", "Fantasy",
    "Horror", "Mahou Shoujo", "Mecha", "Music", "Mystery",
    "Psychological", "Romance", "Sci-Fi", "Slice of Life",
    "Sports", "Supernatural", "Thriller"
]

FORMATS = [
    ("All Formats", ""),
    ("TV Series", "TV"),
    ("Movie", "MOVIE"),
    ("OVA", "OVA"),
    ("ONA", "ONA"),
    ("Special", "SPECIAL")
]

class DiscoverScreen(QWidget):
    media_selected = pyqtSignal(int)  # Emits anilist_id integer

    def __init__(self, parent=None):
        super().__init__(parent)
        self.vm = SearchViewModel(self)
        self.vm.state_changed.connect(self._on_state_changed)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(32, 24, 32, 32)
        layout.setSpacing(16)

        # Header Title
        title_label = QLabel("Discover Anime")
        title_label.setStyleSheet("color: #ffffff; font-size: 24px; font-weight: bold;")
        layout.addWidget(title_label)

        # Controls Header Container (Fixed height layout)
        controls_container = QWidget(self)
        controls_layout = QVBoxLayout(controls_container)
        controls_layout.setContentsMargins(0, 0, 0, 0)
        controls_layout.setSpacing(12)

        # Search Bar
        self.search_input = QLineEdit(controls_container)
        self.search_input.setPlaceholderText("🔍 Search anime by title...")
        self.search_input.setFixedHeight(42)
        self.search_input.textChanged.connect(self._on_search_changed)
        controls_layout.addWidget(self.search_input)

        # Filters Row
        filters_row = QHBoxLayout()
        filters_row.setSpacing(12)

        # Format Dropdown
        self.format_combo = QComboBox(controls_container)
        self.format_combo.clear()
        for label, code in FORMATS:
            self.format_combo.addItem(label, code)
        self.format_combo.currentIndexChanged.connect(self._on_filter_changed)
        filters_row.addWidget(self.format_combo)

        # Sort Dropdown
        self.sort_combo = QComboBox(controls_container)
        self.sort_combo.clear()
        for label, code in [
            ("Sort: Popularity", "POPULARITY_DESC"),
            ("Sort: Score", "SCORE_DESC"),
            ("Sort: Trending", "TRENDING_DESC"),
            ("Sort: Release Date", "START_DATE_DESC")
        ]:
            self.sort_combo.addItem(label, code)

        self.sort_combo.currentIndexChanged.connect(self._on_filter_changed)
        filters_row.addWidget(self.sort_combo)
        filters_row.addStretch()

        controls_layout.addLayout(filters_row)

        # Genre Chips Scrollable Row
        genres_scroll = QScrollArea(controls_container)
        genres_scroll.setFixedHeight(44)
        genres_scroll.setWidgetResizable(True)
        genres_scroll.setFrameShape(QFrame.Shape.NoFrame)
        genres_scroll.setStyleSheet("background: transparent; border: none;")
        genres_scroll.viewport().setStyleSheet("background: transparent; border: none;")
        genres_scroll.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        genres_scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)

        genres_widget = QWidget()
        genres_widget.setStyleSheet("background: transparent;")
        genres_layout = QHBoxLayout(genres_widget)
        genres_layout.setContentsMargins(0, 0, 0, 0)
        genres_layout.setSpacing(8)

        self.genre_buttons = {}
        for genre in GENRES:
            btn = QPushButton(genre, genres_widget)
            btn.setProperty("class", "chip")
            btn.setCheckable(True)
            btn.setCursor(Qt.CursorShape.PointingHandCursor)
            btn.clicked.connect(self._on_genre_toggled)
            genres_layout.addWidget(btn)
            self.genre_buttons[genre] = btn

        genres_scroll.setWidget(genres_widget)
        controls_layout.addWidget(genres_scroll)

        layout.addWidget(controls_container)

        # Poster Grid Scroll Area
        self.scroll_area = QScrollArea(self)
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")
        self.scroll_area.verticalScrollBar().valueChanged.connect(self._on_scroll_changed)


        self.grid_container = QWidget()
        self.grid_container.setStyleSheet("background: transparent;")
        self.grid_layout = QGridLayout(self.grid_container)
        self.grid_layout.setContentsMargins(0, 12, 0, 12)
        self.grid_layout.setSpacing(16)

        self.scroll_area.setWidget(self.grid_container)
        layout.addWidget(self.scroll_area)

        # Debounce timer
        self.search_timer = QTimer(self)
        self.search_timer.setSingleShot(True)
        self.search_timer.timeout.connect(self._execute_search)

        # Initial search execution
        self._execute_search()

    def set_genre_filter(self, genre_name: str):
        if genre_name in self.genre_buttons:
            for btn in self.genre_buttons.values():
                btn.setChecked(False)
            self.genre_buttons[genre_name].setChecked(True)
            self._execute_search()

    def _on_search_changed(self, text: str):
        self.search_timer.start(350)

    def _on_genre_toggled(self):
        self._execute_search()

    def _on_filter_changed(self, idx):
        self._execute_search()

    def _execute_search(self):
        query = self.search_input.text().strip()
        selected_genres = [g for g, btn in self.genre_buttons.items() if btn.isChecked()]
        format_code = self.format_combo.currentData()
        sort_code = self.sort_combo.currentData()

        self.vm.set_query(query)
        self.vm.set_genres(selected_genres)
        self.vm.set_format(format_code)
        self.vm.set_sort(sort_code)

    def _on_scroll_changed(self, value):
        max_val = self.scroll_area.verticalScrollBar().maximum()
        if value >= max_val - 200:
            self.vm.load_next_page()

    def _on_state_changed(self, state: dict):
        # Clear previous grid items
        while self.grid_layout.count() > 0:
            child = self.grid_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        results = state["results"]
        cols = 5
        for i, media in enumerate(results):
            row = i // cols
            col = i % cols
            card = AnimeCard(media=media, parent=self.grid_container)
            card.card_clicked.connect(self._on_card_clicked)
            self.grid_layout.addWidget(card, row, col)

        if state.get("is_capped") and len(results) > 0:
            last_row = (len(results) - 1) // cols + 1
            cap_label = QLabel("Showing top search results", self.grid_container)
            cap_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
            cap_label.setStyleSheet("color: rgba(255, 255, 255, 0.4); font-size: 12px; margin-top: 16px;")
            self.grid_layout.addWidget(cap_label, last_row, 0, 1, cols)

    def _on_card_clicked(self, media):
        anilist_id = media.id if hasattr(media, 'id') else getattr(media, 'anilist_id', 0)
        if anilist_id:
            self.media_selected.emit(int(anilist_id))

