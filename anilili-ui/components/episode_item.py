from PyQt6.QtCore import pyqtSignal, Qt
from PyQt6.QtGui import QPixmap, QImage
from PyQt6.QtWidgets import QFrame, QHBoxLayout, QLabel, QPushButton, QWidget
from components.shimmer_loader import ShimmerImageWidget

class EpisodeItemWidget(QFrame):
    play_clicked = pyqtSignal(object)
    watched_toggled = pyqtSignal(object, bool)  # (episode, new_watched_state)

    def __init__(self, episode, is_watched=False, parent=None):
        super().__init__(parent)
        self.episode = episode
        self.is_watched = is_watched
        self.setFixedHeight(56)
        self.setCursor(Qt.CursorShape.PointingHandCursor)

        self._normal_pixmap = None
        self._blurred_pixmap = None
        self._is_blurred = False

        self._apply_style()

        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 6, 12, 6)
        layout.setSpacing(12)

        # 16:9 Thumbnail (80x45px)
        self.thumb_container = QWidget(self)
        self.thumb_container.setFixedSize(80, 45)
        thumb_layout = QHBoxLayout(self.thumb_container)
        thumb_layout.setContentsMargins(0, 0, 0, 0)

        self.thumb_widget = ShimmerImageWidget(width=80, height=45, border_radius=6, parent=self.thumb_container)
        self.thumb_widget.image_loaded.connect(self._on_image_loaded)

        if hasattr(episode, 'image') and episode.image:
            self.thumb_widget.load_url(episode.image)

        thumb_layout.addWidget(self.thumb_widget)
        layout.addWidget(self.thumb_container)

        # Episode Number (accent color, bold)
        ep_num_str = f"Ep {int(episode.number)}" if episode.number.is_integer() else f"Ep {episode.number}"
        self.ep_num_label = QLabel(ep_num_str)
        self.ep_num_label.setFixedWidth(48)
        self.ep_num_label.setStyleSheet("color: #8979F2; font-size: 13px; font-weight: bold;")
        layout.addWidget(self.ep_num_label)

        # Title Label (1 line truncated)
        title_str = episode.title or f"Episode {episode.number}"
        self.title_label = QLabel(title_str)
        self.title_label.setStyleSheet("color: rgba(255,255,255,0.75); font-size: 13px;")
        layout.addWidget(self.title_label, stretch=1)

        # Filler Badge
        if hasattr(episode, 'filler') and episode.filler:
            filler_badge = QLabel("FILLER")
            filler_badge.setStyleSheet("""
                QLabel {
                    background-color: rgba(186, 117, 23, 0.2);
                    color: #ba7517;
                    font-size: 10px;
                    font-weight: bold;
                    padding: 2px 6px;
                    border-radius: 4px;
                    border: 1px solid rgba(186, 117, 23, 0.4);
                }
            """)
            layout.addWidget(filler_badge)

        # Spoiler Blur Eye Toggle Button
        self.eye_btn = QPushButton("👁", self)
        self.eye_btn.setFixedSize(24, 24)
        self.eye_btn.setToolTip("Toggle Spoiler Blur")
        self.eye_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.eye_btn.setStyleSheet("""
            QPushButton {
                background: transparent;
                color: rgba(255,255,255,0.5);
                border: none;
                font-size: 12px;
            }
            QPushButton:hover {
                color: #ffffff;
            }
        """)
        self.eye_btn.clicked.connect(self._toggle_blur)
        layout.addWidget(self.eye_btn)

        # Interactive Watched Toggle Button
        self.watch_toggle_btn = QPushButton("✓ Watched" if is_watched else "○ Mark Watched", self)
        self.watch_toggle_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.watch_toggle_btn.setFixedHeight(24)
        self._update_watch_toggle_style()
        self.watch_toggle_btn.clicked.connect(self._on_watched_toggled)
        layout.addWidget(self.watch_toggle_btn)

        # Action Buttons: Play + Download
        self.play_btn = QPushButton("▶")
        self.play_btn.setFixedSize(28, 28)
        self.play_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.play_btn.setStyleSheet("""
            QPushButton {
                background-color: #8979F2;
                color: #ffffff;
                border: none;
                border-radius: 14px;
                font-size: 11px;
            }
            QPushButton:hover {
                background-color: #9d91f5;
            }
        """)
        self.play_btn.clicked.connect(lambda: self.play_clicked.emit(self.episode))
        layout.addWidget(self.play_btn)

        self.dl_btn = QPushButton("↓")
        self.dl_btn.setFixedSize(28, 28)
        self.dl_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.dl_btn.setStyleSheet("""
            QPushButton {
                background-color: rgba(255,255,255,0.08);
                color: rgba(255,255,255,0.7);
                border: none;
                border-radius: 14px;
                font-size: 11px;
            }
            QPushButton:hover {
                background-color: #8979F2;
                color: #ffffff;
            }
        """)
        layout.addWidget(self.dl_btn)

    def _on_watched_toggled(self):
        self.is_watched = not self.is_watched
        self.watch_toggle_btn.setText("✓ Watched" if self.is_watched else "○ Mark Watched")
        self._update_watch_toggle_style()
        self._apply_style()
        self.watched_toggled.emit(self.episode, self.is_watched)

    def _update_watch_toggle_style(self):
        if self.is_watched:
            self.watch_toggle_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(29, 158, 117, 0.15);
                    color: #1d9e75;
                    border: 1px solid rgba(29, 158, 117, 0.4);
                    border-radius: 4px;
                    padding: 2px 8px;
                    font-size: 11px;
                    font-weight: 600;
                }
                QPushButton:hover {
                    background-color: rgba(29, 158, 117, 0.3);
                }
            """)
        else:
            self.watch_toggle_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(255, 255, 255, 0.05);
                    color: rgba(255, 255, 255, 0.5);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 4px;
                    padding: 2px 8px;
                    font-size: 11px;
                    font-weight: 600;
                }
                QPushButton:hover {
                    background-color: rgba(255, 255, 255, 0.1);
                    color: rgba(255, 255, 255, 0.8);
                }
            """)

    def set_download_state(self, state: str, progress: float = 0.0):
        if state == "IDLE":
            self.dl_btn.setText("↓")
            self.dl_btn.setToolTip("Download Episode")
            self.dl_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(255,255,255,0.08);
                    color: rgba(255,255,255,0.7);
                    border: none;
                    border-radius: 14px;
                    font-size: 11px;
                }
                QPushButton:hover {
                    background-color: #8979F2;
                    color: #ffffff;
                }
            """)
        elif state == "QUEUED":
            self.dl_btn.setText("🕒")
            self.dl_btn.setToolTip("Download Queued")
            self.dl_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(137, 121, 242, 0.2);
                    color: #8979F2;
                    border: 1px solid rgba(137, 121, 242, 0.4);
                    border-radius: 14px;
                    font-size: 10px;
                }
            """)
        elif state == "DOWNLOADING":
            pct = int(progress * 100)
            self.dl_btn.setText(f"{pct}%")
            self.dl_btn.setToolTip(f"Downloading: {pct}%")
            self.dl_btn.setStyleSheet("""
                QPushButton {
                    background-color: #8979F2;
                    color: #ffffff;
                    border: none;
                    border-radius: 14px;
                    font-size: 9px;
                    font-weight: bold;
                }
            """)
        elif state == "SAVED":
            self.dl_btn.setText("✓")
            self.dl_btn.setToolTip("Downloaded & Saved")
            self.dl_btn.setStyleSheet("""
                QPushButton {
                    background-color: #2e7d32;
                    color: #ffffff;
                    border: none;
                    border-radius: 14px;
                    font-size: 11px;
                    font-weight: bold;
                }
            """)
        elif state == "ERROR":
            self.dl_btn.setText("×")
            self.dl_btn.setToolTip("Download Error")
            self.dl_btn.setStyleSheet("""
                QPushButton {
                    background-color: #d32f2f;
                    color: #ffffff;
                    border: none;
                    border-radius: 14px;
                    font-size: 11px;
                    font-weight: bold;
                }
            """)
        elif state == "CONVERTING":
            self.dl_btn.setText("⚡")
            self.dl_btn.setToolTip("Converting to MP4...")
            self.dl_btn.setStyleSheet("""
                QPushButton {
                    background-color: #f57c00;
                    color: #ffffff;
                    border: none;
                    border-radius: 14px;
                    font-size: 10px;
                }
            """)


    def _on_image_loaded(self, pixmap: QPixmap):
        self._normal_pixmap = pixmap
        # Pre-compute fast scale-blur pixmap (10% scale down with FastTransformation, scale up with SmoothTransformation)
        small = pixmap.scaled(8, 5, Qt.AspectRatioMode.IgnoreAspectRatio, Qt.TransformationMode.FastTransformation)
        self._blurred_pixmap = small.scaled(80, 45, Qt.AspectRatioMode.IgnoreAspectRatio, Qt.TransformationMode.SmoothTransformation)

    def _toggle_blur(self):
        if not self._normal_pixmap or not self._blurred_pixmap:
            return
        self._is_blurred = not self._is_blurred
        if self._is_blurred:
            self.thumb_widget.set_pixmap(self._blurred_pixmap)
            self.eye_btn.setStyleSheet("color: #8979F2; background: transparent; border: none; font-size: 12px;")
        else:
            self.thumb_widget.set_pixmap(self._normal_pixmap)
            self.eye_btn.setStyleSheet("color: rgba(255,255,255,0.5); background: transparent; border: none; font-size: 12px;")

    def _apply_style(self):
        if self.is_watched:
            self.setStyleSheet("""
                QFrame {
                    background-color: rgba(255,255,255,0.04);
                    border-radius: 8px;
                    border: 1px solid rgba(255,255,255,0.04);
                }
                QFrame:hover {
                    background-color: #141416;
                    border-color: rgba(137,121,242,0.3);
                }
            """)
        else:
            self.setStyleSheet("""
                QFrame {
                    background-color: #0e0e10;
                    border-radius: 8px;
                    border: 1px solid rgba(255,255,255,0.05);
                }
                QFrame:hover {
                    background-color: #141416;
                    border-color: rgba(137,121,242,0.3);
                }
            """)


