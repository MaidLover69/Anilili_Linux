from PyQt6.QtCore import pyqtSignal, Qt, QTimer
from PyQt6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QFrame, QStackedWidget
from components.shimmer_loader import ShimmerImageWidget

class HeroSlide(QFrame):
    watch_clicked = pyqtSignal(object)
    detail_clicked = pyqtSignal(object)

    def __init__(self, media, parent=None):
        super().__init__(parent)
        self.media = media
        self.setFixedHeight(300)
        self.setStyleSheet("background-color: #0e0e10; border-radius: 16px;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # Background banner image
        banner_url = media.banner_image or media.cover_image.extra_large or media.cover_image.large
        self.image_widget = ShimmerImageWidget(width=1000, height=300, border_radius=16, parent=self)
        if banner_url:
            self.image_widget.load_url(banner_url)

        layout.addWidget(self.image_widget)

        # Dark gradient overlay with text + buttons
        overlay = QVBoxLayout(self.image_widget)
        overlay.setContentsMargins(32, 32, 32, 32)
        overlay.addStretch()

        self.text_container = QWidget(self.image_widget)
        self.text_container.setStyleSheet("background: transparent; border: none;")
        text_layout = QVBoxLayout(self.text_container)
        text_layout.setContentsMargins(0, 0, 0, 0)
        text_layout.setSpacing(8)

        # Stats info string
        fmt = media.format or "TV"
        score = f"★ {media.average_score / 10.0:.1f}" if media.average_score else ""
        episodes = f"{media.episodes} Episodes" if media.episodes else ""
        stats_parts = [p for p in [fmt, score, episodes] if p]
        stats_str = "  •  ".join(stats_parts)

        stats_label = QLabel(stats_str)
        stats_label.setStyleSheet("color: #a99ef5; font-size: 13px; font-weight: 600; background: transparent;")
        text_layout.addWidget(stats_label)

        # Title
        title_text = media.display_title() if hasattr(media, 'display_title') else media.title.preferred()
        title_label = QLabel(title_text)
        title_label.setWordWrap(True)
        title_label.setStyleSheet("color: #ffffff; font-size: 24px; font-weight: bold; background: transparent;")
        text_layout.addWidget(title_label)

        # Action Buttons
        btn_box = QHBoxLayout()
        btn_box.setSpacing(12)

        watch_btn = QPushButton("▶  Watch Now")
        watch_btn.setProperty("class", "primary")
        watch_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        watch_btn.clicked.connect(lambda: self.watch_clicked.emit(self.media))
        btn_box.addWidget(watch_btn)

        add_btn = QPushButton("+  Details")
        add_btn.setProperty("class", "secondary")
        add_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        add_btn.clicked.connect(lambda: self.detail_clicked.emit(self.media))
        btn_box.addWidget(add_btn)

        btn_box.addStretch()
        text_layout.addLayout(btn_box)

        overlay.addWidget(self.text_container)

    def resizeEvent(self, event):
        self.image_widget.setFixedSize(self.width(), self.height())
        if hasattr(self, 'text_container'):
            self.text_container.setMaximumWidth(int(self.width() * 0.6))
        super().resizeEvent(event)



class HeroBanner(QWidget):
    media_clicked = pyqtSignal(object)
    watch_clicked = pyqtSignal(object)
    details_clicked = pyqtSignal(object)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFixedHeight(320)
        self._slides_data = []
        self._current_index = 0

        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)
        self.layout.setSpacing(8)

        self.stacked_widget = QStackedWidget(self)
        self.layout.addWidget(self.stacked_widget)

        # Dot indicators bar
        self.dots_layout = QHBoxLayout()
        self.dots_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.dots_layout.setSpacing(6)
        self.layout.addLayout(self.dots_layout)

        # Auto-scroll timer (7s)
        self.timer = QTimer(self)
        self.timer.setInterval(7000)
        self.timer.timeout.connect(self._next_slide)

    def set_items(self, media_list):
        self.timer.stop()
        self._slides_data = media_list[:5]  # Top 5 items

        # Clear stacked widget
        while self.stacked_widget.count() > 0:
            widget = self.stacked_widget.widget(0)
            self.stacked_widget.removeWidget(widget)
            widget.deleteLater()

        # Clear dots
        while self.dots_layout.count() > 0:
            child = self.dots_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        if not self._slides_data:
            return

        # Add slides
        for media in self._slides_data:
            slide = HeroSlide(media, self)
            slide.watch_clicked.connect(self._on_slide_clicked)
            slide.detail_clicked.connect(self._on_slide_clicked)
            self.stacked_widget.addWidget(slide)

        # Add dots
        for i in range(len(self._slides_data)):
            dot = QLabel("•")
            dot.setStyleSheet("color: rgba(255,255,255,0.3); font-size: 20px;")
            self.dots_layout.addWidget(dot)

        self._current_index = 0
        self._update_dots()
        if len(self._slides_data) > 1:
            self.timer.start()

    def _on_slide_clicked(self, media):
        self.media_clicked.emit(media)
        self.watch_clicked.emit(media)
        self.details_clicked.emit(media)

    def _next_slide(self):
        if not self._slides_data:
            return
        self._current_index = (self._current_index + 1) % len(self._slides_data)
        self.stacked_widget.setCurrentIndex(self._current_index)
        self._update_dots()

    def _update_dots(self):
        for i in range(self.dots_layout.count()):
            dot = self.dots_layout.itemAt(i).widget()
            if i == self._current_index:
                dot.setStyleSheet("color: #8979F2; font-size: 24px;")
            else:
                dot.setStyleSheet("color: rgba(255,255,255,0.3); font-size: 20px;")

    def enterEvent(self, event):
        self.timer.stop()
        super().enterEvent(event)

    def leaveEvent(self, event):
        if len(self._slides_data) > 1:
            self.timer.start()
        super().leaveEvent(event)
