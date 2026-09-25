from PyQt6.QtCore import pyqtSignal, Qt, QEvent
from PyQt6.QtWidgets import QWidget, QVBoxLayout, QLabel, QHBoxLayout, QFrame
from components.shimmer_loader import ShimmerImageWidget

class AnimeCard(QFrame):
    card_clicked = pyqtSignal(object)  # Emits Media object

    def __init__(self, media=None, show_progress=False, progress_pct=0.0, progress_text=None, parent=None):
        super().__init__(parent)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setFixedWidth(130)
        self.setProperty("class", "anime-card")
        self._media = media

        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)
        self.layout.setSpacing(6)

        # Image Container
        self.image_widget = ShimmerImageWidget(width=130, height=195, border_radius=12, parent=self)
        self.layout.addWidget(self.image_widget)

        # Overlay badges container inside image_widget
        overlay_layout = QVBoxLayout(self.image_widget)
        overlay_layout.setContentsMargins(6, 6, 6, 6)

        top_bar = QHBoxLayout()
        top_bar.setContentsMargins(0, 0, 0, 0)

        # Format Badge (top-left)
        format_val = getattr(media, 'format', None)
        format_str = format_val if format_val else "TV"
        self.format_label = QLabel(format_str)
        self.format_label.setStyleSheet("""
            QLabel {
                background-color: rgba(14, 14, 16, 0.85);
                color: #a99ef5;
                font-size: 10px;
                font-weight: bold;
                padding: 2px 6px;
                border-radius: 4px;
            }
        """)
        top_bar.addWidget(self.format_label)
        top_bar.addStretch()

        # Rating Badge (top-right)
        score_val = getattr(media, 'average_score', 0) or 0
        if score_val > 0:
            score_str = f"★ {score_val / 10.0:.1f}"
            self.score_label = QLabel(score_str)
            self.score_label.setStyleSheet("""
                QLabel {
                    background-color: rgba(14, 14, 16, 0.85);
                    color: #ba7517;
                    font-size: 10px;
                    font-weight: bold;
                    padding: 2px 6px;
                    border-radius: 4px;
                }
            """)
            top_bar.addWidget(self.score_label)

        overlay_layout.addLayout(top_bar)
        overlay_layout.addStretch()

        # Optional Progress Bar or Text Overlay
        if show_progress and progress_pct > 0.0:
            prog_bar = QFrame(self.image_widget)
            prog_bar.setFixedHeight(3)
            fill_width = int(130 * min(1.0, progress_pct))
            prog_bar.setStyleSheet(f"background-color: #8979F2; border-radius: 1px; width: {fill_width}px;")
            overlay_layout.addWidget(prog_bar)
        
        if progress_text:
            prog_lbl = QLabel(progress_text, self.image_widget)
            prog_lbl.setStyleSheet("""
                QLabel {
                    background-color: rgba(0, 0, 0, 0.7);
                    color: #ffffff;
                    font-size: 10px;
                    font-weight: 500;
                    padding: 2px 4px;
                    border-radius: 4px;
                }
            """)
            overlay_layout.addWidget(prog_lbl, alignment=Qt.AlignmentFlag.AlignLeft)

        # Title Label below card
        if media:
            if hasattr(media, 'display_title'):
                title_text = media.display_title()
            elif isinstance(getattr(media, 'title', None), str):
                title_text = media.title
            elif hasattr(getattr(media, 'title', None), 'preferred'):
                title_text = media.title.preferred()
            else:
                title_text = "Untitled"
        else:
            title_text = "Untitled"

        self.title_label = QLabel(title_text)
        self.title_label.setWordWrap(True)
        self.title_label.setMaximumHeight(36)
        self.title_label.setStyleSheet("""
            QLabel {
                color: #ffffff;
                font-size: 12px;
                font-weight: 500;
            }
        """)
        self.layout.addWidget(self.title_label)

        if media:
            cover_url = None
            if hasattr(media, 'cover_image') and media.cover_image:
                cover_url = media.cover_image.extra_large or media.cover_image.large
            elif hasattr(media, 'cover') and media.cover:
                cover_url = media.cover
            if cover_url:
                self.image_widget.load_url(cover_url)

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton and self._media:
            self.card_clicked.emit(self._media)
        super().mousePressEvent(event)


