from PyQt6.QtCore import pyqtSignal, Qt, QRectF
from PyQt6.QtWidgets import QWidget, QToolTip
from PyQt6.QtGui import QPainter, QColor, QPen, QBrush

class SeekBar(QWidget):
    seek_requested = pyqtSignal(float)  # Emits target position in seconds

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFixedHeight(20)
        self.setMouseTracking(True)
        self.setCursor(Qt.CursorShape.PointingHandCursor)

        self.duration_s = 0.0
        self.position_s = 0.0
        self.buffer_fraction = 0.0  # Default 0.0
        self.skip_times = {}       # { "intro_start": float, "intro_end": float, ... }

        self._is_dragging = False
        self._is_hovered = False
        self._hover_x = 0

    def set_duration(self, duration_s: float):
        self.duration_s = max(0.0, duration_s)
        self.update()

    def set_position(self, position_s: float):
        if not self._is_dragging:
            self.position_s = max(0.0, position_s)
            self.update()

    def set_buffer_fraction(self, fraction: float):
        self.buffer_fraction = max(0.0, min(1.0, fraction))
        self.update()

    def set_skip_times(self, skip_times: dict):
        self.skip_times = skip_times or {}
        self.update()

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        width = self.width()
        height = self.height()
        track_h = 4
        track_y = (height - track_h) // 2

        # 1. Background track
        painter.setPen(Qt.PenStyle.NoPen)
        painter.setBrush(QColor(255, 255, 255, 50))
        painter.drawRoundedRect(0, track_y, width, track_h, 2, 2)

        # 2. Buffer fill (Safe default buffer_fraction=0.0)
        if self.buffer_fraction > 0.0:
            buf_w = int(width * self.buffer_fraction)
            painter.setBrush(QColor(255, 255, 255, 40))
            painter.drawRoundedRect(0, track_y, buf_w, track_h, 2, 2)

        # 3. Progress fill
        if self.duration_s > 0.0:
            prog_ratio = min(1.0, max(0.0, self.position_s / self.duration_s))
            prog_w = int(width * prog_ratio)
            painter.setBrush(QColor("#8979F2"))
            painter.drawRoundedRect(0, track_y, prog_w, track_h, 2, 2)

        # 4. Chapter markers (AniSkip intro/outro boundaries)
        if self.duration_s > 0.0:
            marker_pen = QPen(QColor("#a99ef5"), 2)
            painter.setPen(marker_pen)
            for key in ["intro_start", "intro_end", "outro_start", "outro_end"]:
                val = self.skip_times.get(key)
                if val is not None and 0.0 <= val <= self.duration_s:
                    x = int(width * (val / self.duration_s))
                    painter.drawLine(x, track_y - 2, x, track_y + track_h + 2)

        # 5. Thumb handle on hover or drag
        if (self._is_hovered or self._is_dragging) and self.duration_s > 0.0:
            prog_ratio = min(1.0, max(0.0, self.position_s / self.duration_s))
            thumb_x = int(width * prog_ratio)
            thumb_r = 6
            painter.setPen(Qt.PenStyle.NoPen)
            painter.setBrush(QColor("#ffffff"))
            painter.drawEllipse(thumb_x - thumb_r, (height // 2) - thumb_r, thumb_r * 2, thumb_r * 2)

    def enterEvent(self, event):
        self._is_hovered = True
        self.update()
        super().enterEvent(event)

    def leaveEvent(self, event):
        self._is_hovered = False
        self.update()
        super().leaveEvent(event)

    def mouseMoveEvent(self, event):
        self._hover_x = event.position().x()
        width = self.width()
        if width > 0 and self.duration_s > 0.0:
            hover_pos_s = max(0.0, min(self.duration_s, (self._hover_x / width) * self.duration_s))
            mins = int(hover_pos_s // 60)
            secs = int(hover_pos_s % 60)
            QToolTip.showText(event.globalPosition().toPoint(), f"{mins:02d}:{secs:02d}", self)

        if self._is_dragging and width > 0 and self.duration_s > 0.0:
            self.position_s = max(0.0, min(self.duration_s, (self._hover_x / width) * self.duration_s))
            self.update()

        super().mouseMoveEvent(event)

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton and self.width() > 0 and self.duration_s > 0.0:
            self._is_dragging = True
            pos_x = event.position().x()
            self.position_s = max(0.0, min(self.duration_s, (pos_x / self.width()) * self.duration_s))
            self.update()
        super().mousePressEvent(event)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton and self._is_dragging:
            self._is_dragging = False
            self.seek_requested.emit(self.position_s)
        super().mouseReleaseEvent(event)
