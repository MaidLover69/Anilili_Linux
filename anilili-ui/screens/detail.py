import math
import re
from PyQt6.QtCore import pyqtSignal, Qt
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QFrame,
    QStackedWidget, QSizePolicy, QScrollArea
)
from PyQt6.QtGui import QPixmap, QImage, QColor, QPainter, QLinearGradient
from components.shimmer_loader import ShimmerImageWidget
from components.episode_browser import EpisodeBrowserWidget
from components.horizontal_rail import HorizontalAnimeRail
from viewmodels.detail_vm import DetailViewModel



class RelationMediaItem:
    def __init__(self, node: dict):
        self.id = node["id"]
        self.id_mal = node.get("idMal")
        t_obj = node.get("title", {})
        self.title_str = t_obj.get("english") or t_obj.get("userPreferred") or t_obj.get("romaji") or "Untitled"
        c_obj = node.get("coverImage", {})
        self.cover_url = c_obj.get("extraLarge") or c_obj.get("large")
        self.format = node.get("format")
        self.average_score = node.get("averageScore")
        self.is_adult = False

        # Duck typing cover_image and display_title
        class CoverObj:
            def __init__(self, url):
                self.extra_large = url
                self.large = url

        self.cover_image = CoverObj(self.cover_url)

    def display_title(self):
        return self.title_str


class DetailScreen(QWidget):
    back_clicked = pyqtSignal()
    navigate_discover = pyqtSignal(dict)  # { "genre": str, "studio": str }
    play_episode_requested = pyqtSignal(object)  # EpisodeItem

    def __init__(self, parent=None):
        super().__init__(parent)
        self.vm = DetailViewModel(self)
        self.vm.state_changed.connect(self._on_state_changed)

        self._anilist_id = None
        self._current_details = None
        self._synopsis_expanded = False
        self._full_synopsis = ""
        self._in_watchlist = False


        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # Scroll Area Wrapper
        self.scroll_area = QScrollArea(self)


        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")

        self.container = QWidget()
        self.container.setStyleSheet("background: transparent;")
        self.content_layout = QVBoxLayout(self.container)
        self.content_layout.setContentsMargins(0, 0, 0, 32)
        self.content_layout.setSpacing(24)

        # Top Banner & Header Section
        self.banner_container = QWidget(self.container)
        self.banner_container.setFixedHeight(320)
        self.banner_layout = QVBoxLayout(self.banner_container)
        self.banner_layout.setContentsMargins(0, 0, 0, 0)

        self.banner_image_widget = ShimmerImageWidget(width=1200, height=280, border_radius=0, parent=self.banner_container)
        self.banner_layout.addWidget(self.banner_image_widget)

        # Back Button Overlay (top-left)
        banner_overlay = QVBoxLayout(self.banner_image_widget)
        banner_overlay.setContentsMargins(20, 20, 20, 20)

        top_nav_row = QHBoxLayout()
        self.back_btn = QPushButton("←  Back", self.banner_container)
        self.back_btn.setProperty("class", "secondary")
        self.back_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.back_btn.clicked.connect(self.back_clicked.emit)
        top_nav_row.addWidget(self.back_btn)
        top_nav_row.addStretch()

        banner_overlay.addLayout(top_nav_row)
        banner_overlay.addStretch()

        self.content_layout.addWidget(self.banner_container)

        # Main Information Section (Cover Art + Metadata Header)
        info_section = QWidget(self.container)
        info_layout = QHBoxLayout(info_section)
        info_layout.setContentsMargins(32, 0, 32, 0)
        info_layout.setSpacing(24)

        # Cover Image (130x195px)
        self.cover_widget = ShimmerImageWidget(width=130, height=195, border_radius=12, parent=info_section)
        info_layout.addWidget(self.cover_widget)

        # Metadata Details Column
        meta_col = QVBoxLayout()
        meta_col.setSpacing(8)

        self.title_label = QLabel("Loading...", info_section)
        self.title_label.setWordWrap(True)
        self.title_label.setStyleSheet("color: #ffffff; font-size: 24px; font-weight: bold;")
        meta_col.addWidget(self.title_label)

        self.sub_meta_label = QLabel("", info_section)
        self.sub_meta_label.setStyleSheet("color: rgba(255,255,255,0.65); font-size: 13px;")
        meta_col.addWidget(self.sub_meta_label)

        self.score_label = QLabel("", info_section)
        self.score_label.setStyleSheet("color: #ba7517; font-size: 13px; font-weight: bold;")
        meta_col.addWidget(self.score_label)

        # Action Buttons Row
        action_row = QHBoxLayout()
        action_row.setSpacing(12)

        self.watch_ep_btn = QPushButton("▶ Watch Ep 1")
        self.watch_ep_btn.setProperty("class", "primary")
        self.watch_ep_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.watch_ep_btn.clicked.connect(self._on_watch_btn_clicked)
        action_row.addWidget(self.watch_ep_btn)

        self.watchlist_btn = QPushButton("+ Watchlist")
        self.watchlist_btn.setProperty("class", "secondary")
        self.watchlist_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.watchlist_btn.clicked.connect(self._on_watchlist_btn_clicked)
        action_row.addWidget(self.watchlist_btn)

        self.download_btn = QPushButton("↓ Download All")
        self.download_btn.setProperty("class", "secondary")
        self.download_btn.setEnabled(False)
        action_row.addWidget(self.download_btn)
        action_row.addStretch()

        meta_col.addLayout(action_row)
        info_layout.addLayout(meta_col, stretch=1)

        self.content_layout.addWidget(info_section)

        # Synopsis Section
        synopsis_section = QWidget(self.container)
        synopsis_layout = QVBoxLayout(synopsis_section)
        synopsis_layout.setContentsMargins(32, 0, 32, 0)
        synopsis_layout.setSpacing(6)

        self.synopsis_label = QLabel(synopsis_section)
        self.synopsis_label.setWordWrap(True)
        self.synopsis_label.setStyleSheet("color: rgba(255,255,255,0.75); font-size: 14px; line-height: 1.4;")
        synopsis_layout.addWidget(self.synopsis_label)

        self.show_more_btn = QPushButton("Show more ▼", synopsis_section)
        self.show_more_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.show_more_btn.setStyleSheet("color: #8979F2; background: transparent; border: none; font-size: 12px; font-weight: bold; text-align: left;")
        self.show_more_btn.clicked.connect(self._toggle_synopsis)
        synopsis_layout.addWidget(self.show_more_btn)

        self.content_layout.addWidget(synopsis_section)

        # Genre Chips Section
        self.genres_container = QWidget(self.container)
        genres_outer_layout = QVBoxLayout(self.genres_container)
        genres_outer_layout.setContentsMargins(32, 0, 32, 0)

        self.genres_layout = QHBoxLayout()
        self.genres_layout.setSpacing(8)
        self.genres_layout.setAlignment(Qt.AlignmentFlag.AlignLeft)
        genres_outer_layout.addLayout(self.genres_layout)

        self.content_layout.addWidget(self.genres_container)

        # Separator Line
        line = QFrame(self.container)
        line.setFrameShape(QFrame.Shape.HLine)
        line.setStyleSheet("color: rgba(255,255,255,0.07); margin: 0 32px;")
        self.content_layout.addWidget(line)

        # Episode Browser Widget (Full Width)
        self.ep_browser_wrapper = QWidget(self.container)
        ep_wrapper_layout = QVBoxLayout(self.ep_browser_wrapper)
        ep_wrapper_layout.setContentsMargins(32, 0, 32, 0)

        self.ep_browser = EpisodeBrowserWidget(self.ep_browser_wrapper)
        self.ep_browser.play_episode.connect(self.play_episode_requested.emit)
        self.ep_browser.download_requested.connect(self._on_single_download)
        self.ep_browser.watched_toggled.connect(self._on_episode_watched_toggled)
        ep_wrapper_layout.addWidget(self.ep_browser)

        # Bulk Download Button Row
        dl_all_row = QHBoxLayout()
        dl_all_row.addStretch()
        self.dl_all_btn = QPushButton("↓ Download All", self.ep_browser_wrapper)
        self.dl_all_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.dl_all_btn.setStyleSheet("""
            QPushButton {
                background-color: rgba(137, 121, 242, 0.15);
                color: #8979F2;
                border: 1px solid rgba(137, 121, 242, 0.4);
                border-radius: 6px;
                padding: 6px 12px;
                font-size: 12px;
                font-weight: 600;
            }
            QPushButton:hover {
                background-color: #8979F2;
                color: #ffffff;
            }
        """)
        self.dl_all_btn.clicked.connect(self._on_bulk_download)
        dl_all_row.addWidget(self.dl_all_btn)
        ep_wrapper_layout.addLayout(dl_all_row)

        self.content_layout.addWidget(self.ep_browser_wrapper)


        # Relations Rail
        self.relations_rail_container = QWidget(self.container)
        rel_outer_layout = QVBoxLayout(self.relations_rail_container)
        rel_outer_layout.setContentsMargins(32, 0, 32, 0)

        self.relations_rail = HorizontalAnimeRail("Related Anime", "relations", self.relations_rail_container)
        self.relations_rail.card_clicked.connect(lambda media: self.load_anime(media.id))
        rel_outer_layout.addWidget(self.relations_rail)
        self.relations_rail_container.setVisible(False)

        self.content_layout.addWidget(self.relations_rail_container)

        self.scroll_area.setWidget(self.container)
        layout.addWidget(self.scroll_area)

    def load_anime(self, anilist_id: int):
        self._anilist_id = anilist_id
        self._synopsis_expanded = False
        self._full_synopsis = ""
        self.vm.load(anilist_id)
        from utils.backend_bridge import BackendBridge
        BackendBridge.instance().is_in_watchlist(anilist_id, self._on_watchlist_checked)

    def _on_watchlist_checked(self, success, is_in, err):
        if success and is_in:
            self._in_watchlist = True
            self.watchlist_btn.setText("✓ In Watchlist")
        else:
            self._in_watchlist = False
            self.watchlist_btn.setText("+ Watchlist")


    def _on_state_changed(self, state: dict):
        if state["loading"]:
            self.title_label.setText("Loading details...")
            return

        details = state["details"]
        if not details:
            return

        self._current_details = details

        # Update Title
        title_obj = details.get("title", {})
        title_text = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or "Untitled"
        self.title_label.setText(title_text)

        # Update Sub Metadata (Format · Year · Studio)
        fmt = details.get("format") or "TV"
        year = details.get("seasonYear") or ""
        episodes_cnt = details.get("episodes")
        ep_str = f"{episodes_cnt} episodes" if episodes_cnt else "? episodes"

        # Main Studio
        main_studio = ""
        studios_edges = details.get("studios", {}).get("edges", [])
        for edge in studios_edges:
            if edge.get("isMain"):
                main_studio = edge.get("node", {}).get("name", "")
                break
        if not main_studio and studios_edges:
            main_studio = studios_edges[0].get("node", {}).get("name", "")

        meta_parts = [p for p in [fmt, str(year) if year else "", main_studio, ep_str] if p]
        self.sub_meta_label.setText("  •  ".join(meta_parts))

        # Score & Status
        score = details.get("averageScore")
        status = details.get("status") or "FINISHED"
        score_str = f"★ {score / 10.0:.1f}  ({status})" if score else status
        self.score_label.setText(score_str)

        # Cover Image
        cover_url = details.get("coverImage", {}).get("extraLarge") or details.get("coverImage", {}).get("large")
        if cover_url:
            self.cover_widget.load_url(cover_url)

        # Banner Image (Fast Scale-Blur Fallback if banner missing)
        banner_url = details.get("bannerImage")
        if banner_url:
            self.banner_image_widget.load_url(banner_url)
        elif cover_url:
            self.banner_image_widget.load_url(cover_url)

        # Synopsis
        raw_synopsis = details.get("description") or "No synopsis available."
        clean_synopsis = re.sub(r'<[^>]+>', '', raw_synopsis)
        self._full_synopsis = clean_synopsis
        self._update_synopsis_display()

        # Genre Chips
        while self.genres_layout.count() > 0:
            child = self.genres_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        genres = details.get("genres", [])
        for g in genres:
            chip = QPushButton(g, self.genres_container)
            chip.setProperty("class", "chip")
            chip.setCursor(Qt.CursorShape.PointingHandCursor)
            chip.clicked.connect(lambda _, genre_name=g: self.navigate_discover.emit({"genre": genre_name}))
            self.genres_layout.addWidget(chip)

        # Episodes Browser
        provider_data = state["episodes"]
        konoha_data = state["konoha"]
        watched_data = state["watched"]
        self.ep_browser.set_data(provider_data, konoha_data, watched_data)

        # Update Next Unwatched Episode Button Label
        all_eps = []
        for pdata in provider_data.values():
            all_eps.extend(pdata.get("sub", []))
        next_ep = self.vm.get_next_unwatched_episode(all_eps, watched_data)
        ep_label = int(next_ep) if next_ep.is_integer() else next_ep
        self.watch_ep_btn.setText(f"▶ Watch Ep {ep_label}")

        # Relations Rail (SEQUEL, PREQUEL, SIDE_STORY, SPIN_OFF, PARENT)
        allowed_relations = {"SEQUEL", "PREQUEL", "SIDE_STORY", "SPIN_OFF", "PARENT"}
        relation_edges = details.get("relations", {}).get("edges", [])
        rel_media_list = []
        for edge in relation_edges:
            rel_type = edge.get("relationType")
            if rel_type in allowed_relations:
                node = edge.get("node")
                if node:
                    rel_item = RelationMediaItem(node)
                    rel_media_list.append(rel_item)

        if rel_media_list:
            rail_title = f"Related Anime ({len(rel_media_list)})" if len(rel_media_list) > 10 else "Related Anime"
            self.relations_rail.title_label.setText(rail_title.upper())
            self.relations_rail.set_items(rel_media_list[:10])
            self.relations_rail_container.setVisible(True)
        else:
            self.relations_rail_container.setVisible(False)

    def _update_synopsis_display(self):
        full_synopsis = getattr(self, '_full_synopsis', '')
        if self._synopsis_expanded:
            self.synopsis_label.setText(full_synopsis)
            self.show_more_btn.setText("Show less ▲")
        else:
            if len(full_synopsis) > 220:
                self.synopsis_label.setText(full_synopsis[:220] + "...")
                self.show_more_btn.setVisible(True)
                self.show_more_btn.setText("Show more ▼")
            else:
                self.synopsis_label.setText(full_synopsis)
                self.show_more_btn.setVisible(False)

    def _toggle_synopsis(self):
        self._synopsis_expanded = not self._synopsis_expanded
        self._update_synopsis_display()

    def _on_watch_btn_clicked(self):
        if self._current_details:
            anilist_id = self._current_details["id"]
            mal_id = self._current_details.get("idMal")
            title_obj = self._current_details.get("title", {})
            anime_title = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or ""
            
            all_eps = []
            for pdata in self.vm._state["episodes"].values():
                all_eps.extend(pdata.get("sub", []))
            next_ep_num = self.vm.get_next_unwatched_episode(all_eps, self.vm._state["watched"])

            play_info = {
                "anilist_id": anilist_id,
                "mal_id": mal_id,
                "episode_number": next_ep_num,
                "category": "sub",
                "anime_title": anime_title,
                "episode_title": f"Episode {next_ep_num}"
            }
            print(f"[DetailScreen] Watch button clicked: emitting play_episode_requested for ep {next_ep_num}")
            self.play_episode_requested.emit(play_info)

    def _on_episode_watched_toggled(self, episode, new_watched_state: bool):
        ep_num = getattr(episode, 'number', 0.0)
        self.vm.toggle_watched(ep_num, new_watched_state)

    def _on_single_download(self, episode):
        if not self._current_details:
            return
        anilist_id = self._current_details["id"]
        mal_id = self._current_details.get("idMal")
        title_obj = self._current_details.get("title", {})
        series_title = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or "Untitled"
        cover_url = self._current_details.get("coverImage", {}).get("extraLarge") or self._current_details.get("coverImage", {}).get("large")

        # Resolve episode source async then call DownloadManager
        from utils.backend_bridge import BackendBridge
        from utils.download_manager import DownloadManager
        import asyncio

        bridge = BackendBridge.instance()
        ep_num = getattr(episode, "number", 1.0)

        def _on_sources(streams):
            if streams:
                DownloadManager.instance().start_download(
                    episode=episode,
                    stream_item=streams[0],
                    series_title=series_title,
                    series_cover=cover_url,
                    anilist_id=anilist_id,
                    quality="720p"
                )

        bridge.episode_sources_loaded.connect(_on_sources)
        bridge.load_episode_sources(anilist_id, mal_id, ep_num, "sub")

    def _on_bulk_download(self):
        if not self._current_details:
            return
        from components.bulk_download_dialog import BulkDownloadDialog
        anilist_id = self._current_details["id"]
        mal_id = self._current_details.get("idMal")
        title_obj = self._current_details.get("title", {})
        series_title = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or "Untitled"
        cover_url = self._current_details.get("coverImage", {}).get("extraLarge") or self._current_details.get("coverImage", {}).get("large")

        episodes = self.ep_browser._get_active_episodes()
        if not episodes:
            return

        dialog = BulkDownloadDialog(
            episodes=episodes,
            anilist_id=anilist_id,
            mal_id=mal_id,
            series_title=series_title,
            series_cover=cover_url,
            parent=self
        )
        dialog.exec()

    def _on_watchlist_btn_clicked(self):
        if not self._current_details:
            return

        from utils.backend_bridge import BackendBridge
        anilist_id = self._current_details["id"]
        title_obj = self._current_details.get("title", {})
        series_title = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or "Untitled"
        cover_url = self._current_details.get("coverImage", {}).get("extraLarge") or self._current_details.get("coverImage", {}).get("large")
        fmt = self._current_details.get("format")
        score = self._current_details.get("averageScore")

        if self._in_watchlist:
            BackendBridge.instance().remove_from_watchlist(anilist_id)
            self._in_watchlist = False
            self.watchlist_btn.setText("+ Watchlist")
        else:
            BackendBridge.instance().add_to_watchlist(anilist_id, series_title, cover_url, fmt, score)
            self._in_watchlist = True
            self.watchlist_btn.setText("✓ In Watchlist")

