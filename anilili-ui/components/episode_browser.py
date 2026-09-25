import math
from PyQt6.QtCore import pyqtSignal, Qt
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QComboBox,
    QLineEdit, QFrame, QStackedWidget, QGridLayout, QScrollArea, QListView
)
from components.episode_item import EpisodeItemWidget



class EpisodeBrowserWidget(QWidget):
    play_episode = pyqtSignal(object)
    download_requested = pyqtSignal(object)  # Emits episode object
    watched_toggled = pyqtSignal(object, bool)  # (episode, new_watched_state)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._provider_data = {}
        self._konoha_map = {}
        self._watched_list = []
        self._download_items_map = {}  # ep.number -> ep_item widget

        self._current_category = "sub"
        self._current_provider = None
        self._current_chunk_idx = 0
        self._filter_text = ""
        self._view_mode = "list" # "list" or "grid"

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        # Header Row 1: Episodes Label + Sub/Dub Toggle + View Mode Toggle
        top_row = QHBoxLayout()
        self.title_label = QLabel("EPISODES (0)")
        self.title_label.setStyleSheet("color: #ffffff; font-size: 16px; font-weight: bold;")
        top_row.addWidget(self.title_label)

        top_row.addStretch()

        # Category (Sub/Dub) Toggle
        self.sub_btn = QPushButton("Sub")
        self.sub_btn.setProperty("class", "chip")
        self.sub_btn.setProperty("selected", "true")
        self.sub_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.sub_btn.clicked.connect(lambda: self._set_category("sub"))
        top_row.addWidget(self.sub_btn)

        self.dub_btn = QPushButton("Dub")
        self.dub_btn.setProperty("class", "chip")
        self.dub_btn.setProperty("selected", "false")
        self.dub_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.dub_btn.clicked.connect(lambda: self._set_category("dub"))
        top_row.addWidget(self.dub_btn)

        top_row.addSpacing(12)

        # List / Grid Toggle
        self.list_mode_btn = QPushButton("☰")
        self.list_mode_btn.setFixedSize(32, 32)
        self.list_mode_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.list_mode_btn.setStyleSheet("background-color: #8979F2; color: white; border-radius: 6px;")
        self.list_mode_btn.clicked.connect(lambda: self._set_view_mode("list"))
        top_row.addWidget(self.list_mode_btn)

        self.grid_mode_btn = QPushButton("☷")
        self.grid_mode_btn.setFixedSize(32, 32)
        self.grid_mode_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.grid_mode_btn.setStyleSheet("background-color: rgba(255,255,255,0.05); color: white; border-radius: 6px;")
        self.grid_mode_btn.clicked.connect(lambda: self._set_view_mode("grid"))
        top_row.addWidget(self.grid_mode_btn)

        layout.addLayout(top_row)

        # Header Row 2: Provider Dropdown + Instant Ep Search + Chunk Selector
        controls_row = QHBoxLayout()
        controls_row.setSpacing(12)

        combo_style = """
            QComboBox {
                background-color: #1c1c20;
                color: #ffffff;
                border: 1px solid rgba(137, 121, 242, 0.4);
                border-radius: 6px;
                padding: 4px 10px;
                font-size: 13px;
                font-weight: 600;
            }
            QComboBox:hover {
                background-color: #24242a;
                border-color: #8979F2;
            }
            QComboBox QAbstractItemView {
                background-color: #1c1c20;
                color: #ffffff;
                selection-background-color: #8979F2;
                selection-color: #ffffff;
                border: 1px solid rgba(137, 121, 242, 0.5);
                border-radius: 6px;
                outline: 0;
                padding: 4px;
            }
            QComboBox QAbstractItemView::item {
                min-height: 28px;
                color: #ffffff;
                background-color: #1c1c20;
                padding: 4px 8px;
            }
            QComboBox QAbstractItemView::item:selected {
                background-color: #8979F2;
                color: #ffffff;
            }
        """

        self.provider_combo = QComboBox(self)
        self.provider_combo.setView(QListView(self.provider_combo))
        self.provider_combo.setStyleSheet(combo_style)
        self.provider_combo.currentIndexChanged.connect(self._on_provider_changed)
        controls_row.addWidget(self.provider_combo)

        self.chunk_combo = QComboBox(self)
        self.chunk_combo.setView(QListView(self.chunk_combo))
        self.chunk_combo.setStyleSheet(combo_style)
        self.chunk_combo.currentIndexChanged.connect(self._on_chunk_changed)
        self.chunk_combo.setVisible(False)
        controls_row.addWidget(self.chunk_combo)

        self.search_input = QLineEdit(self)
        self.search_input.setPlaceholderText("🔍 Filter episodes...")
        self.search_input.setFixedHeight(34)
        self.search_input.textChanged.connect(self._on_search_text_changed)
        controls_row.addWidget(self.search_input, stretch=1)

        layout.addLayout(controls_row)

        # Container Widget for Episode Items (expands naturally inside DetailScreen scroll area)
        self.episodes_container = QWidget(self)
        self.episodes_container.setStyleSheet("background: transparent;")
        self.episodes_layout = QVBoxLayout(self.episodes_container)
        self.episodes_layout.setContentsMargins(0, 0, 0, 0)
        self.episodes_layout.setSpacing(6)
        self.episodes_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        layout.addWidget(self.episodes_container)

    def _close_combo_popups(self):
        if hasattr(self, 'provider_combo'):
            self.provider_combo.hidePopup()
        if hasattr(self, 'chunk_combo'):
            self.chunk_combo.hidePopup()

    def set_data(self, provider_data: dict, konoha_list: list, watched_list: list):
        self._provider_data = provider_data or {}
        self._watched_list = watched_list or []
        self._konoha_map = {}
        for item in (konoha_list or []):
            if hasattr(item, 'number'):
                self._konoha_map[round(item.number)] = item

        self._populate_providers()

    def _populate_providers(self):
        self.provider_combo.blockSignals(True)
        self.provider_combo.clear()

        has_dub = False
        for name, data in self._provider_data.items():
            sub_cnt = len(data.get("sub", []))
            dub_cnt = len(data.get("dub", []))
            if dub_cnt > 0:
                has_dub = True
            label = f"{name} ({sub_cnt} sub / {dub_cnt} dub)"
            self.provider_combo.addItem(label, name)

        self.dub_btn.setVisible(has_dub)
        self.provider_combo.blockSignals(False)

        if self.provider_combo.count() > 0:
            self._current_provider = self.provider_combo.currentData()
            self._render_episodes()

    def _set_category(self, cat: str):
        self._current_category = cat
        self.sub_btn.setProperty("selected", "true" if cat == "sub" else "false")
        self.dub_btn.setProperty("selected", "true" if cat == "dub" else "false")
        self.sub_btn.style().unpolish(self.sub_btn); self.sub_btn.style().polish(self.sub_btn)
        self.dub_btn.style().unpolish(self.dub_btn); self.dub_btn.style().polish(self.dub_btn)
        self._render_episodes()

    def _set_view_mode(self, mode: str):
        self._view_mode = mode
        self.list_mode_btn.setStyleSheet(
            "background-color: #8979F2; color: white; border-radius: 6px;" if mode == "list"
            else "background-color: rgba(255,255,255,0.05); color: white; border-radius: 6px;"
        )
        self.grid_mode_btn.setStyleSheet(
            "background-color: #8979F2; color: white; border-radius: 6px;" if mode == "grid"
            else "background-color: rgba(255,255,255,0.05); color: white; border-radius: 6px;"
        )
        self._render_episodes()

    def _on_provider_changed(self, idx):
        if idx >= 0:
            self._current_provider = self.provider_combo.itemData(idx)
            self._current_chunk_idx = 0
            self._render_episodes()

    def _on_chunk_changed(self, idx):
        if idx >= 0:
            self._current_chunk_idx = idx
            self._render_episodes()

    def _on_search_text_changed(self, text):
        self._filter_text = text.strip().lower()
        self._render_episodes()

    def _get_active_episodes(self):
        if not self._current_provider or self._current_provider not in self._provider_data:
            return []
        p_data = self._provider_data[self._current_provider]
        raw_list = p_data.get(self._current_category, [])

        # Enrich with Konoha titles/thumbnails if raw title is missing
        enriched = []
        for item in raw_list:
            ep_num = round(item.number)
            if ep_num in self._konoha_map:
                k_item = self._konoha_map[ep_num]
                if not item.title:
                    item.title = k_item.title
                if not item.image:
                    item.image = k_item.image
                item.filler = k_item.filler
            enriched.append(item)

        # Filter by search text
        if self._filter_text:
            enriched = [
                ep for ep in enriched
                if (ep.title and self._filter_text in ep.title.lower()) or
                   (self._filter_text in str(ep.number))
            ]

        return enriched

    def _render_episodes(self):
        # Disconnect signals and clear items map for clean chunk switching
        for item in self._download_items_map.values():
            try:
                item.play_clicked.disconnect()
            except Exception:
                pass
            try:
                item.dl_btn.clicked.disconnect()
            except Exception:
                pass
            try:
                item.watched_toggled.disconnect()
            except Exception:
                pass
        self._download_items_map.clear()

        # Re-create layout based on view_mode (List vs Grid)
        if self.episodes_container.layout() is not None:
            QWidget().setLayout(self.episodes_container.layout())

        if self._view_mode == "grid":
            self.episodes_layout = QGridLayout(self.episodes_container)
            self.episodes_layout.setContentsMargins(0, 0, 0, 0)
            self.episodes_layout.setSpacing(10)
            self.episodes_layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        else:
            self.episodes_layout = QVBoxLayout(self.episodes_container)
            self.episodes_layout.setContentsMargins(0, 0, 0, 0)
            self.episodes_layout.setSpacing(6)
            self.episodes_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        episodes = self._get_active_episodes()
        self.title_label.setText(f"EPISODES ({len(episodes)})")

        if not episodes:
            empty_lbl = QLabel("No episodes found for this selection")
            empty_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
            empty_lbl.setStyleSheet("color: rgba(255,255,255,0.4); font-size: 14px; padding: 24px;")
            if self._view_mode == "grid":
                self.episodes_layout.addWidget(empty_lbl, 0, 0, 1, 5)
            else:
                self.episodes_layout.addWidget(empty_lbl)
            self.chunk_combo.setVisible(False)
            return

        # Setup Chunk Selector (max 100 widgets per chunk)
        chunk_size = 100
        num_chunks = math.ceil(len(episodes) / chunk_size)

        if num_chunks > 1:
            self.chunk_combo.blockSignals(True)
            self.chunk_combo.clear()
            for c in range(num_chunks):
                start_ep = c * chunk_size + 1
                end_ep = min((c + 1) * chunk_size, len(episodes))
                self.chunk_combo.addItem(f"Episodes {start_ep}-{end_ep}", c)
            self.chunk_combo.setCurrentIndex(self._current_chunk_idx)
            self.chunk_combo.blockSignals(False)
            self.chunk_combo.setVisible(True)
        else:
            self.chunk_combo.setVisible(False)

        # Slice active chunk (max 100)
        start_idx = self._current_chunk_idx * chunk_size
        end_idx = min(start_idx + chunk_size, len(episodes))
        chunk_episodes = episodes[start_idx:end_idx]

        self._download_items_map.clear()

        # Render active chunk items
        cols = 5
        for i, ep in enumerate(chunk_episodes):
            # Float-safe watched check: abs(ep.number - w) < 0.01
            is_w = any(abs(ep.number - w) < 0.01 for w in self._watched_list)
            ep_item = EpisodeItemWidget(ep, is_watched=is_w, parent=self.episodes_container)
            ep_item.play_clicked.connect(self.play_episode.emit)
            ep_item.dl_btn.clicked.connect(lambda checked, episode_obj=ep: self.download_requested.emit(episode_obj))
            ep_item.watched_toggled.connect(self.watched_toggled.emit)
            
            self._download_items_map[round(ep.number, 2)] = ep_item
            if self._view_mode == "grid":
                row = i // cols
                col = i % cols
                self.episodes_layout.addWidget(ep_item, row, col)
            else:
                self.episodes_layout.addWidget(ep_item)


