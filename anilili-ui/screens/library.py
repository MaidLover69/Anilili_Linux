from PyQt6.QtCore import pyqtSignal, Qt
from PyQt6.QtGui import QPixmap, QPainter, QPainterPath
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QComboBox,
    QLineEdit, QFrame, QFileDialog, QStackedLayout, QScrollArea
)
from auth import AniListAuthManager, MalAuthManager
from viewmodels.library_vm import LibraryViewModel, TAB_STATUS_MAP
from components.anime_card import AnimeCard



class LibraryScreen(QWidget):
    media_selected = pyqtSignal(int)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.vm = LibraryViewModel(self)

        # Outer Layout
        self.outer_layout = QVBoxLayout(self)
        self.outer_layout.setContentsMargins(24, 24, 24, 24)

        # Create Logged-in & Logged-out Widgets
        self.logged_in_widget = QWidget(self)
        self.logged_out_widget = QWidget(self)

        self.outer_layout.addWidget(self.logged_in_widget)
        self.outer_layout.addWidget(self.logged_out_widget)

        self._setup_logged_in_ui()
        self._setup_logged_out_ui()

        # Connect VM Signals
        self.vm.library_loaded.connect(self._render_grid)
        self.vm.viewer_loaded.connect(self._render_viewer)
        self.vm.error.connect(self._show_error)
        self.vm.reload_requested.connect(self._update_view_state)

        # Connect Auth Managers signals for dialog cancel / state change
        AniListAuthManager.instance().auth_state_changed.connect(lambda _: self._update_view_state())
        AniListAuthManager.instance().auth_cancelled.connect(self._update_view_state)
        MalAuthManager.instance().auth_state_changed.connect(lambda _: self._update_view_state())
        MalAuthManager.instance().auth_cancelled.connect(self._update_view_state)

        self._update_view_state()

    def _update_view_state(self):
        anilist_auth = AniListAuthManager.instance().is_authenticated()
        mal_auth = MalAuthManager.instance().is_authenticated()

        if anilist_auth or mal_auth:
            self.logged_out_widget.setVisible(False)
            self.logged_in_widget.setVisible(True)
            self.vm.load(self._active_tab_label)
        else:
            self.logged_in_widget.setVisible(False)
            self.logged_out_widget.setVisible(True)

    def _setup_logged_in_ui(self):
        layout = QVBoxLayout(self.logged_in_widget)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(16)

        # Top Header (Avatar + Username + Stats + Logout)
        header_row = QHBoxLayout()
        
        self.avatar_lbl = QLabel(self)
        self.avatar_lbl.setFixedSize(48, 48)
        self.avatar_lbl.setStyleSheet("background-color: #141416; border-radius: 24px;")
        header_row.addWidget(self.avatar_lbl)

        info_col = QVBoxLayout()
        self.username_lbl = QLabel("Anime Fan", self)
        self.username_lbl.setStyleSheet("color: #ffffff; font-size: 16px; font-weight: bold;")
        self.stats_lbl = QLabel("0 anime · 0 episodes watched", self)
        self.stats_lbl.setStyleSheet("color: rgba(255,255,255,0.6); font-size: 12px;")
        info_col.addWidget(self.username_lbl)
        info_col.addWidget(self.stats_lbl)
        header_row.addLayout(info_col, stretch=1)

        self.logout_btn = QPushButton("Logout", self)
        self.logout_btn.setProperty("class", "secondary")
        self.logout_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.logout_btn.clicked.connect(self._on_logout)
        header_row.addWidget(self.logout_btn)

        layout.addLayout(header_row)

        # Tab Bar: [Watching] [Planning] [Completed] [Paused] [Dropped] [Re-watching]
        self.tab_row = QHBoxLayout()
        self.tab_buttons = {}
        self._active_tab_label = "Watching"

        for label in ["Watching", "Planning", "Completed", "Paused", "Dropped", "Re-watching"]:
            btn = QPushButton(label, self)
            btn.setCursor(Qt.CursorShape.PointingHandCursor)
            btn.clicked.connect(lambda checked, l=label: self._on_tab_click(l))
            self.tab_row.addWidget(btn)
            self.tab_buttons[label] = btn

        self._style_tabs()
        layout.addLayout(self.tab_row)

        # Controls Row (Sort + Format + Search)
        controls_row = QHBoxLayout()
        
        self.sort_combo = QComboBox(self)
        self.sort_combo.addItems(["Title", "Score", "Updated"])
        self.sort_combo.setStyleSheet("""
            QComboBox {
                background-color: #1a1a1d;
                color: #ffffff;
                border: 1px solid rgba(255,255,255,0.1);
                border-radius: 6px;
                padding: 6px 12px;
            }
        """)
        controls_row.addWidget(self.sort_combo)

        self.format_combo = QComboBox(self)
        self.format_combo.addItems(["All Formats", "TV", "Movie", "OVA", "ONA"])
        self.format_combo.setStyleSheet("""
            QComboBox {
                background-color: #1a1a1d;
                color: #ffffff;
                border: 1px solid rgba(255,255,255,0.1);
                border-radius: 6px;
                padding: 6px 12px;
            }
        """)
        controls_row.addWidget(self.format_combo)

        self.search_input = QLineEdit(self)
        self.search_input.setPlaceholderText("Filter list...")
        self.search_input.setStyleSheet("""
            QLineEdit {
                background-color: #1a1a1d;
                color: #ffffff;
                border: 1px solid rgba(255,255,255,0.1);
                border-radius: 6px;
                padding: 6px 12px;
            }
        """)
        self.search_input.textChanged.connect(self.vm.filter_entries)
        controls_row.addWidget(self.search_input, stretch=1)

        layout.addLayout(controls_row)

        # Main Content Grid Scroll Area
        self.scroll_area = QScrollArea(self)


        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")

        self.grid_container = QWidget()
        self.grid_container.setStyleSheet("background: transparent;")
        self.grid_layout = QHBoxLayout(self.grid_container)
        self.grid_layout.setContentsMargins(0, 0, 0, 0)
        self.grid_layout.setSpacing(12)
        self.grid_layout.setAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignTop)

        self.scroll_area.setWidget(self.grid_container)
        layout.addWidget(self.scroll_area, stretch=1)

        # Real Downloads Section
        self.downloads_frame = QFrame(self)
        self.downloads_frame.setStyleSheet("""
            QFrame {
                background-color: #141416;
                border-radius: 8px;
                border: 1px solid rgba(255,255,255,0.06);
            }
        """)
        self.downloads_layout = QVBoxLayout(self.downloads_frame)
        self.downloads_layout.setContentsMargins(16, 16, 16, 16)
        self.downloads_layout.setSpacing(12)
        layout.addWidget(self.downloads_frame)

        # Connect Download signals for real-time updates
        from utils.download_manager import DownloadManager
        DownloadManager.instance().download_complete.connect(lambda _: self._refresh_downloads())
        DownloadManager.instance().download_error.connect(lambda _, __: self._refresh_downloads())
        self.vm.bridge.downloads_loaded.connect(self._render_downloads)


    def _setup_logged_out_ui(self):
        layout = QVBoxLayout(self.logged_out_widget)
        layout.setContentsMargins(40, 40, 40, 40)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        # AniList Auth Box
        anilist_title = QLabel("AniList", self)
        anilist_title.setStyleSheet("color: #8979F2; font-size: 24px; font-weight: bold;")
        layout.addWidget(anilist_title, alignment=Qt.AlignmentFlag.AlignCenter)

        connect_anilist_btn = QPushButton("Connect AniList", self)
        connect_anilist_btn.setProperty("class", "primary")
        connect_anilist_btn.setFixedSize(200, 40)
        connect_anilist_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        connect_anilist_btn.clicked.connect(lambda: AniListAuthManager.instance().start_auth(self))
        layout.addWidget(connect_anilist_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addSpacing(24)

        # MAL Auth Box
        mal_title = QLabel("MyAnimeList", self)
        mal_title.setStyleSheet("color: #2e51a2; font-size: 24px; font-weight: bold;")
        layout.addWidget(mal_title, alignment=Qt.AlignmentFlag.AlignCenter)

        connect_mal_btn = QPushButton("Connect MyAnimeList", self)
        connect_mal_btn.setProperty("class", "secondary")
        connect_mal_btn.setFixedSize(200, 40)
        connect_mal_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        connect_mal_btn.clicked.connect(lambda: MalAuthManager.instance().start_auth(self))
        layout.addWidget(connect_mal_btn, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addSpacing(24)

        # OR Separator
        sep_lbl = QLabel("─── OR ───", self)
        sep_lbl.setStyleSheet("color: rgba(255,255,255,0.38); font-size: 12px;")
        layout.addWidget(sep_lbl, alignment=Qt.AlignmentFlag.AlignCenter)

        layout.addSpacing(16)

        # Import MAL XML Button
        import_xml_btn = QPushButton("Import MAL XML", self)
        import_xml_btn.setStyleSheet("""
            QPushButton {
                background: transparent;
                color: rgba(255,255,255,0.65);
                border: 1px solid rgba(255,255,255,0.2);
                border-radius: 8px;
                padding: 8px 16px;
                font-weight: 600;
            }
            QPushButton:hover {
                color: #ffffff;
                border-color: #8979F2;
            }
        """)
        import_xml_btn.setFixedSize(200, 40)
        import_xml_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        import_xml_btn.clicked.connect(self._on_import_xml)
        layout.addWidget(import_xml_btn, alignment=Qt.AlignmentFlag.AlignCenter)

    def _on_tab_click(self, label: str):
        self._active_tab_label = label
        self._style_tabs()
        self.vm.switch_tab(label)

    def _style_tabs(self):
        for label, btn in self.tab_buttons.items():
            if label == self._active_tab_label:
                btn.setStyleSheet("""
                    QPushButton {
                        color: #8979F2;
                        font-weight: bold;
                        border: none;
                        border-bottom: 2px solid #8979F2;
                        background: transparent;
                        padding: 6px 12px;
                    }
                """)
            else:
                btn.setStyleSheet("""
                    QPushButton {
                        color: rgba(255,255,255,0.5);
                        border: none;
                        background: transparent;
                        padding: 6px 12px;
                    }
                    QPushButton:hover {
                        color: #ffffff;
                    }
                """)

    def _render_grid(self, entries):
        # Clear existing layout items
        while self.grid_layout.count() > 0:
            child = self.grid_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        for entry in entries:
            progress_str = f"Ep {entry.progress}/{entry.total_episodes}" if getattr(entry, 'total_episodes', None) else f"Ep {entry.progress}"
            
            # Wrap MediaListEntry so AnimeCard can extract fields
            card = AnimeCard(
                media=entry,
                show_progress=True,
                progress_pct=(entry.progress / entry.total_episodes) if getattr(entry, 'total_episodes', None) else 0.0,
                progress_text=progress_str,
                parent=self.grid_container
            )
            card.card_clicked.connect(lambda e: self.media_selected.emit(e.id))
            self.grid_layout.addWidget(card)

    def _render_viewer(self, viewer):
        if not viewer:
            return
        self.username_lbl.setText(viewer.name)
        self.stats_lbl.setText(f"{viewer.anime_count} anime · {viewer.episodes_watched} episodes watched")

    def _on_logout(self):
        self.vm.logout_anilist()
        self.vm.logout_mal()

    def _on_import_xml(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Select MAL Export XML", "", "XML Files (*.xml *.xml.gz)"
        )
        if file_path:
            self.vm.import_mal_xml(file_path)

    def _refresh_downloads(self):
        self.vm.bridge.load_downloads()

    def _render_downloads(self, records):
        # Clear downloads layout
        while self.downloads_layout.count() > 0:
            child = self.downloads_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        header_row = QHBoxLayout()
        title_lbl = QLabel(f"DOWNLOADS ({len(records)})", self.downloads_frame)
        title_lbl.setStyleSheet("color: #ffffff; font-size: 14px; font-weight: bold;")
        header_row.addWidget(title_lbl)
        header_row.addStretch()
        self.downloads_layout.addLayout(header_row)

        if not records:
            empty_lbl = QLabel("No downloads yet", self.downloads_frame)
            empty_lbl.setStyleSheet("color: rgba(255,255,255,0.4); font-size: 13px; padding: 12px;")
            empty_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
            self.downloads_layout.addWidget(empty_lbl)
            return

        # Group downloads by anilist_id
        grouped = {}
        for r in records:
            grouped.setdefault(r.anilist_id, []).append(r)

        for anilist_id, ep_records in grouped.items():
            first_rec = ep_records[0]
            
            # Disk usage calculation (Feedback #7)
            total_bytes = 0
            has_unknown = False
            for rec in ep_records:
                if rec.file_size and rec.file_size > 0:
                    total_bytes += rec.file_size
                else:
                    # Fallback to quality size estimate
                    est = 300 * 1024 * 1024
                    if rec.quality == "1080p":
                        est = 550 * 1024 * 1024
                    elif rec.quality == "480p":
                        est = 150 * 1024 * 1024
                    elif rec.quality == "360p":
                        est = 80 * 1024 * 1024
                    total_bytes += est
                    has_unknown = True

            size_mb = total_bytes / (1024 * 1024)
            size_str = f"~{size_mb / 1024:.1f} GB" if size_mb >= 1024 else f"~{int(size_mb)} MB"
            if not total_bytes and not has_unknown:
                size_str = "Unknown"

            row_frame = QFrame(self.downloads_frame)
            row_frame.setStyleSheet("""
                QFrame {
                    background-color: #1a1a1d;
                    border-radius: 6px;
                    border: 1px solid rgba(255,255,255,0.05);
                }
            """)
            r_layout = QHBoxLayout(row_frame)
            r_layout.setContentsMargins(12, 8, 12, 8)
            r_layout.setSpacing(12)

            # Info text
            info_lbl = QLabel(f"<b>{first_rec.series_title}</b> — {len(ep_records)} episode(s) ({size_str})")
            info_lbl.setStyleSheet("color: #ffffff; font-size: 13px;")
            r_layout.addWidget(info_lbl, stretch=1)

            # Actions: Export & Delete
            exp_btn = QPushButton("Export MP4", row_frame)
            exp_btn.setCursor(Qt.CursorShape.PointingHandCursor)
            exp_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(137, 121, 242, 0.15);
                    color: #8979F2;
                    border: 1px solid rgba(137, 121, 242, 0.3);
                    border-radius: 4px;
                    padding: 4px 10px;
                    font-size: 11px;
                }
                QPushButton:hover {
                    background-color: #8979F2;
                    color: #ffffff;
                }
            """)
            exp_btn.clicked.connect(lambda checked, recs=ep_records: self._export_series(recs))
            r_layout.addWidget(exp_btn)

            del_btn = QPushButton("🗑", row_frame)
            del_btn.setCursor(Qt.CursorShape.PointingHandCursor)
            del_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(211, 47, 47, 0.15);
                    color: #d32f2f;
                    border: 1px solid rgba(211, 47, 47, 0.3);
                    border-radius: 4px;
                    padding: 4px 8px;
                    font-size: 11px;
                }
                QPushButton:hover {
                    background-color: #d32f2f;
                    color: #ffffff;
                }
            """)
            del_btn.clicked.connect(lambda checked, recs=ep_records: self._delete_series(recs))
            r_layout.addWidget(del_btn)

            self.downloads_layout.addWidget(row_frame)

    def _export_series(self, records: list):
        from utils.episode_exporter import EpisodeExporter
        import asyncio

        output_dir = str(Path.home() / "Downloads" / "Anilili")
        exporter = EpisodeExporter()

        for rec in records:
            if rec.status == "SAVED":
                asyncio.create_task(exporter.export_to_mp4(rec.id, output_dir))

    def _delete_series(self, records: list):
        from utils.download_manager import DownloadManager
        for rec in records:
            DownloadManager.instance().delete_download(rec.id)
        self._refresh_downloads()

    def _show_error(self, err_msg: str):
        print(f"[LibraryScreen] Error: {err_msg}")

