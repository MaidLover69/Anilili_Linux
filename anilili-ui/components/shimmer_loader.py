import httpx
import asyncio
from PyQt6.QtCore import QObject, pyqtSignal, QTimer, QPropertyAnimation, pyqtProperty, Qt
from PyQt6.QtGui import QPixmap, QImage, QPainter, QLinearGradient, QColor
from PyQt6.QtWidgets import QWidget, QLabel, QVBoxLayout

class ImageFetcher(QObject):
    image_ready = pyqtSignal(bytes, str)
    image_failed = pyqtSignal(str)

    _instance = None

    @classmethod
    def instance(cls):
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def fetch(self, url: str):
        if not url:
            return
        asyncio.create_task(self._download(url))

    async def _download(self, url: str):
        try:
            async with httpx.AsyncClient(timeout=10.0, follow_redirects=True) as client:
                resp = await client.get(url)
                if resp.status_code == 200:
                    self.image_ready.emit(resp.content, url)
                else:
                    self.image_failed.emit(url)
        except Exception:
            self.image_failed.emit(url)


class ShimmerImageWidget(QWidget):
    image_loaded = pyqtSignal(object)  # Emits QPixmap when image is loaded

    def __init__(self, width: int = 130, height: int = 195, border_radius: int = 12, parent=None):
        super().__init__(parent)
        self.setFixedSize(width, height)
        self._border_radius = border_radius
        self._pixmap = None
        self._url = None
        self._offset = 0.0

        # Shimmer timer
        self._shimmer_timer = QTimer(self)
        self._shimmer_timer.setInterval(30)
        self._shimmer_timer.timeout.connect(self._update_shimmer)
        self._shimmer_timer.start()

        # Connect fetcher signals
        fetcher = ImageFetcher.instance()
        fetcher.image_ready.connect(self._on_image_ready)
        fetcher.image_failed.connect(self._on_image_failed)

    def load_url(self, url: str):
        if not url or url == self._url:
            return
        self._url = url
        self._pixmap = None
        self.update()
        ImageFetcher.instance().fetch(url)

    def set_pixmap(self, pixmap: QPixmap):
        self._pixmap = pixmap
        if self._shimmer_timer.isActive():
            self._shimmer_timer.stop()
        self.update()

    def _on_image_ready(self, data: bytes, url: str):
        if url != self._url:
            return
        # Construct QPixmap strictly on main GUI thread from image bytes
        img = QImage()
        if img.loadFromData(data):
            scaled_pixmap = QPixmap.fromImage(img).scaled(
                self.width(),
                self.height(),
                Qt.AspectRatioMode.KeepAspectRatioByExpanding,
                Qt.TransformationMode.SmoothTransformation
            )
            self.set_pixmap(scaled_pixmap)
            self.image_loaded.emit(scaled_pixmap)


    def _on_image_failed(self, url: str):
        if url == self._url and self._shimmer_timer.isActive():
            self._shimmer_timer.stop()
            self.update()

    def _update_shimmer(self):
        self._offset += 0.03
        if self._offset > 1.5:
            self._offset = -0.5
        self.update()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        if self._pixmap and not self._pixmap.isNull():
            # Draw actual image with rounded corners
            painter.setClipPath(self._clip_path())
            painter.drawPixmap(0, 0, self._pixmap)
        else:
            # Draw shimmer animation
            painter.setClipPath(self._clip_path())
            grad = QLinearGradient(0, 0, self.width(), self.height())
            base = QColor("#0e0e10")
            highlight = QColor("#1f1f26")

            pos = self._offset
            grad.setColorAt(max(0.0, min(1.0, pos - 0.2)), base)
            grad.setColorAt(max(0.0, min(1.0, pos)), highlight)
            grad.setColorAt(max(0.0, min(1.0, pos + 0.2)), base)

            painter.fillRect(self.rect(), grad)

    def _clip_path(self):
        from PyQt6.QtGui import QPainterPath
        path = QPainterPath()
        path.addRoundedRect(0, 0, self.width(), self.height(), self._border_radius, self._border_radius)
        return path
