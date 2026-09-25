from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QCheckBox,
    QComboBox, QSlider, QListWidget, QListWidgetItem, QFrame, QScrollArea,
    QFileDialog, QMessageBox
)
from settings.store import SettingsStore
from utils.backend_bridge import BackendBridge
from components.update_dialog import UpdateDialog

ALL_PROVIDERS = [
    ("bonk", "Bonk (Miruro Pipe Native Leader)"),
    ("anibd", "AniBD (Anivexa Native Leader)"),
    ("senshi", "Senshi (Anivexa Fast Leader)"),
    ("kaa", "KickAssAnime (Fast API Provider)"),
    ("anikoto", "AniKoto (MegaPlay Provider)"),
    ("allanime", "AllAnime (Standard Scraper)"),
    ("animekai", "AnimeKai (Anikai.cc Provider)"),
    ("anidbapp", "AniDB App (Standard Scraper)"),
    ("reanime", "ReAnime (Flixcloud Scraper)"),
    ("anizone", "AniZone (Scraper)"),
    ("animegg", "AnimeGG (Scraper)"),
    ("anineko", "AniNeko (Scraper)"),
    ("2dhive", "2Dhive (Scraper)"),
    ("rareanimes", "RareAnimes (Scraper)")
]

class SettingsScreen(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.store = SettingsStore.instance()
        self.bridge = BackendBridge.instance()

        layout = QVBoxLayout(self)
        layout.setContentsMargins(32, 24, 32, 32)
        layout.setSpacing(20)

        # Header Title
        header_title = QLabel("Settings")
        header_title.setStyleSheet("color: #ffffff; font-size: 24px; font-weight: bold;")
        layout.addWidget(header_title)

        # Scroll Area
        scroll_area = QScrollArea(self)
        scroll_area.setWidgetResizable(True)
        scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        scroll_area.setStyleSheet("background: transparent;")

        container = QWidget()
        container.setStyleSheet("background: transparent;")
        c_layout = QVBoxLayout(container)
        c_layout.setContentsMargins(0, 0, 0, 0)
        c_layout.setSpacing(24)

        # 1. PLAYBACK
        c_layout.addWidget(self._create_section_header("PLAYBACK"))
        playback_card = self._create_card()
        pb_layout = QVBoxLayout(playback_card)
        pb_layout.setSpacing(12)

        self.chk_autoplay = self._add_checkbox("Autoplay next episode automatically", "autoplay", pb_layout)
        self.chk_autoskip = self._add_checkbox("Auto-skip intro & outro (AniSkip)", "auto_skip_intro_outro", pb_layout)
        self.chk_dub = self._add_checkbox("Prefer Dubbed audio", "prefer_dub", pb_layout)
        self.chk_sub_dub = self._add_checkbox("Show subtitles with Dubbed audio", "subtitles_with_dub", pb_layout)
        self.chk_gestures = self._add_checkbox("Enable player mouse/touch gestures", "player_gestures", pb_layout)

        # Default Quality Combo
        q_row = QHBoxLayout()
        q_label = QLabel("Default Video Quality:")
        q_label.setStyleSheet("color: #ffffff; font-size: 13px;")
        q_row.addWidget(q_label)
        self.combo_quality = QComboBox(playback_card)
        for label, val in [("Highest", "highest"), ("1080p", "1080p"), ("720p", "720p"), ("480p", "480p"), ("Lowest", "lowest")]:
            self.combo_quality.addItem(label, val)
        current_q = self.store.get("default_quality", "highest")
        idx = self.combo_quality.findData(current_q)
        if idx >= 0:
            self.combo_quality.setCurrentIndex(idx)
        self.combo_quality.currentIndexChanged.connect(lambda: self.store.set("default_quality", self.combo_quality.currentData()))
        q_row.addWidget(self.combo_quality)
        q_row.addStretch()
        pb_layout.addLayout(q_row)

        c_layout.addWidget(playback_card)

        # 2. CAPTIONS
        c_layout.addWidget(self._create_section_header("CAPTIONS"))
        captions_card = self._create_card()
        cap_layout = QVBoxLayout(captions_card)
        cap_layout.setSpacing(14)

        # Live Caption Preview Box
        preview_container = QFrame(captions_card)
        preview_container.setFixedHeight(90)
        preview_container.setStyleSheet("background-color: #0b0b0d; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);")
        prev_layout = QVBoxLayout(preview_container)
        prev_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self.caption_preview_label = QLabel("The quick brown fox jumps over the lazy dog")
        prev_layout.addWidget(self.caption_preview_label)
        cap_layout.addWidget(preview_container)

        # Scale slider
        self.slider_scale = self._add_slider("Text Scale:", 50, 200, "caption_text_scale", 100, "%", cap_layout)
        self.slider_scale.valueChanged.connect(self._update_caption_preview)

        # Text Color Combo
        col_row = QHBoxLayout()
        col_label = QLabel("Text Color:")
        col_label.setStyleSheet("color: #ffffff; font-size: 13px;")
        col_row.addWidget(col_label)
        self.combo_cap_color = QComboBox(captions_card)
        for label, val in [("White", "white"), ("Yellow", "#ffff55"), ("Cyan", "#55ffff"), ("Green", "#55ff55")]:
            self.combo_cap_color.addItem(label, val)
        self.combo_cap_color.currentIndexChanged.connect(self._on_cap_color_changed)
        col_row.addWidget(self.combo_cap_color)
        col_row.addStretch()
        cap_layout.addLayout(col_row)

        # Background Color Combo
        bg_col_row = QHBoxLayout()
        bg_col_label = QLabel("Background Color:")
        bg_col_label.setStyleSheet("color: #ffffff; font-size: 13px;")
        bg_col_row.addWidget(bg_col_label)
        self.combo_cap_bg = QComboBox(captions_card)
        for label, val in [("Black", "black"), ("Dark Gray", "#222222"), ("Transparent", "transparent")]:
            self.combo_cap_bg.addItem(label, val)
        self.combo_cap_bg.currentIndexChanged.connect(self._on_cap_bg_changed)
        bg_col_row.addWidget(self.combo_cap_bg)
        bg_col_row.addStretch()
        cap_layout.addLayout(bg_col_row)

        # Opacity slider
        self.slider_op = self._add_slider("Background Opacity:", 0, 100, "caption_background_opacity", 60, "%", cap_layout)
        self.slider_op.valueChanged.connect(self._update_caption_preview)

        # Bold check
        self.chk_bold = self._add_checkbox("Bold Caption Text", "caption_bold_text", cap_layout)
        self.chk_bold.toggled.connect(self._update_caption_preview)

        c_layout.addWidget(captions_card)
        self._update_caption_preview()

        # 3. DOWNLOADS
        c_layout.addWidget(self._create_section_header("DOWNLOADS"))
        dl_card = self._create_card()
        dl_layout = QVBoxLayout(dl_card)
        dl_layout.setSpacing(12)

        dl_q_row = QHBoxLayout()
        dl_q_label = QLabel("Download Quality Preference:")
        dl_q_label.setStyleSheet("color: #ffffff; font-size: 13px;")
        dl_q_row.addWidget(dl_q_label)
        self.combo_dl_q = QComboBox(dl_card)
        for label, val in [("Best Available", "best"), ("1080p", "1080p"), ("720p", "720p"), ("480p", "480p")]:
            self.combo_dl_q.addItem(label, val)
        self.combo_dl_q.currentIndexChanged.connect(lambda: self.store.set("download_quality", self.combo_dl_q.currentData()))
        dl_q_row.addWidget(self.combo_dl_q)
        dl_q_row.addStretch()
        dl_layout.addLayout(dl_q_row)

        c_layout.addWidget(dl_card)

        # 4. PROVIDERS (DRAG TO REORDER PRIORITY)
        c_layout.addWidget(self._create_section_header("PROVIDERS & SERVER PRIORITY"))
        prov_card = self._create_card()
        prov_layout = QVBoxLayout(prov_card)
        prov_layout.setSpacing(12)

        prov_info = QLabel("Drag items to reorder streaming server priority (top has highest priority):")
        prov_info.setStyleSheet("color: rgba(255, 255, 255, 0.7); font-size: 12px;")
        prov_layout.addWidget(prov_info)

        self.provider_list = QListWidget(prov_card)
        self.provider_list.setDragDropMode(QListWidget.DragDropMode.InternalMove)
        self.provider_list.setFixedHeight(180)
        self.provider_list.setStyleSheet("""
            QListWidget {
                background-color: #1a1a1d;
                color: #ffffff;
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 8px;
                padding: 4px;
            }
            QListWidget::item {
                padding: 6px 10px;
                border-radius: 4px;
            }
            QListWidget::item:hover {
                background-color: rgba(137, 121, 242, 0.2);
            }
            QListWidget::item:selected {
                background-color: #8979F2;
            }
        """)

        # Load saved provider priority or default catalog
        saved_p_str = self.store.get("server_priority", "")
        saved_order = [s.strip() for s in saved_p_str.split(",") if s.strip()] if saved_p_str else []
        dict_providers = dict(ALL_PROVIDERS)
        
        # Populate saved order first, then remaining
        added_keys = set()
        for key in saved_order:
            if key in dict_providers:
                item = QListWidgetItem(f"≡  {dict_providers[key]}")
                item.setData(Qt.ItemDataRole.UserRole, key)
                self.provider_list.addItem(item)
                added_keys.add(key)
        for key, name in ALL_PROVIDERS:
            if key not in added_keys:
                item = QListWidgetItem(f"≡  {name}")
                item.setData(Qt.ItemDataRole.UserRole, key)
                self.provider_list.addItem(item)

        self.provider_list.model().rowsMoved.connect(self._on_providers_reordered)
        prov_layout.addWidget(self.provider_list)

        c_layout.addWidget(prov_card)

        # 5. APP & UPDATES
        c_layout.addWidget(self._create_section_header("APP & UPDATES"))
        app_card = self._create_card()
        app_layout = QVBoxLayout(app_card)
        app_layout.setSpacing(12)

        app_info_row = QHBoxLayout()
        ver_label = QLabel("Installed Version: Anilili Linux v1.0.0")

        ver_label.setStyleSheet("color: rgba(255, 255, 255, 0.7); font-size: 13px;")
        app_info_row.addWidget(ver_label)
        app_info_row.addStretch()

        self.check_update_btn = QPushButton("🔄 Check for Updates Now", app_card)
        self.check_update_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.check_update_btn.setStyleSheet("""
            QPushButton {
                background-color: #8979F2;
                color: #ffffff;
                border-radius: 6px;
                padding: 8px 16px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #9b8df4;
            }
        """)
        self.check_update_btn.clicked.connect(self._on_check_update_clicked)
        app_info_row.addWidget(self.check_update_btn)

        app_layout.addLayout(app_info_row)
        c_layout.addWidget(app_card)

        scroll_area.setWidget(container)
        layout.addWidget(scroll_area)

    def _create_section_header(self, text: str) -> QLabel:
        lbl = QLabel(text)
        lbl.setStyleSheet("color: #8979F2; font-size: 13px; font-weight: bold; letter-spacing: 1px;")
        return lbl

    def _create_card(self) -> QFrame:
        card = QFrame()
        card.setStyleSheet("QFrame { background-color: #1a1a1d; border-radius: 10px; border: 1px solid rgba(255,255,255,0.06); padding: 12px; }")
        return card

    def _add_checkbox(self, text: str, key: str, parent_layout: QVBoxLayout) -> QCheckBox:
        chk = QCheckBox(text)
        chk.setCursor(Qt.CursorShape.PointingHandCursor)
        chk.setStyleSheet("QCheckBox { color: #ffffff; font-size: 13px; } QCheckBox::indicator { width: 18px; height: 18px; }")
        chk.setChecked(self.store.get(key, False))
        chk.toggled.connect(lambda val, k=key: self.store.set(k, val))
        parent_layout.addWidget(chk)
        return chk

    def _add_slider(self, label_text: str, min_val: int, max_val: int, key: str, default: int, unit: str, parent_layout: QVBoxLayout) -> QSlider:
        row = QHBoxLayout()
        lbl = QLabel(label_text)
        lbl.setStyleSheet("color: #ffffff; font-size: 13px;")
        row.addWidget(lbl)

        val_lbl = QLabel(f"{self.store.get(key, default)}{unit}")
        val_lbl.setStyleSheet("color: #8979F2; font-weight: bold; font-size: 13px;")

        slider = QSlider(Qt.Orientation.Horizontal)
        slider.setRange(min_val, max_val)
        slider.setValue(int(self.store.get(key, default)))
        slider.valueChanged.connect(lambda val, k=key, vl=val_lbl, u=unit: (self.store.set(k, val), vl.setText(f"{val}{u}")))

        row.addWidget(slider, stretch=1)
        row.addWidget(val_lbl)
        parent_layout.addLayout(row)
        return slider

    def _on_cap_color_changed(self):
        val = self.combo_cap_color.currentData()
        if val:
            self.store.set("caption_text_color", val)
            self._update_caption_preview()

    def _on_cap_bg_changed(self):
        val = self.combo_cap_bg.currentData()
        if val:
            self.store.set("caption_background_color", val)
            self._update_caption_preview()

    def _update_caption_preview(self):
        scale = self.store.get("caption_text_scale", 100)
        color = self.store.get("caption_text_color", "white")
        bg_col = self.store.get("caption_background_color", "black")
        op = self.store.get("caption_background_opacity", 60)
        bold = "bold" if self.store.get("caption_bold_text", True) else "normal"

        font_size = int(14 * (scale / 100.0))
        css = f"""
            QLabel {{
                color: {color};
                font-size: {font_size}px;
                font-weight: {bold};
                background-color: rgba(0, 0, 0, {op / 100.0});
                padding: 4px 10px;
                border-radius: 4px;
            }}
        """
        self.caption_preview_label.setStyleSheet(css)

    def _on_providers_reordered(self, parent, start, end, destination, row):
        keys = []
        for i in range(self.provider_list.count()):
            item = self.provider_list.item(i)
            k = item.data(Qt.ItemDataRole.UserRole)
            if k:
                keys.append(k)
        self.store.set("server_priority", ",".join(keys))

    def _on_check_update_clicked(self):
        self.check_update_btn.setEnabled(False)
        self.check_update_btn.setText("Checking...")

        def _cb(success, info, err):
            self.check_update_btn.setEnabled(True)
            self.check_update_btn.setText("🔄 Check for Updates Now")
            if success and info:
                dialog = UpdateDialog(info, self)
                dialog.exec()
            else:
                QMessageBox.information(self, "No Update Available", "You are running the latest version of Anilili Linux (v1.0.0).")

        self.bridge.check_for_update(_cb)
