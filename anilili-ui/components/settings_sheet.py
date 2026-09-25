from PyQt6.QtCore import pyqtSignal, Qt, QPropertyAnimation, QRect
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QComboBox, QFrame
)

class PlayerSettingsSheet(QFrame):
    quality_changed = pyqtSignal(object)
    speed_changed = pyqtSignal(float)
    sub_track_changed = pyqtSignal(int)
    audio_track_changed = pyqtSignal(int)
    sub_delay_changed = pyqtSignal(float)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("settings_sheet")
        self.setFixedHeight(300)
        self.setStyleSheet("""
            QFrame#settings_sheet {
                background-color: rgba(14, 14, 16, 0.95);
                border-top-left-radius: 16px;
                border-top-right-radius: 16px;
                border: 1px solid rgba(255,255,255,0.1);
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 16, 24, 24)
        layout.setSpacing(16)

        # Header Row
        header_row = QHBoxLayout()
        title_lbl = QLabel("PLAYER SETTINGS")
        title_lbl.setStyleSheet("color: #ffffff; font-size: 14px; font-weight: bold;")
        header_row.addWidget(title_lbl)
        header_row.addStretch()

        close_btn = QPushButton("✕")
        close_btn.setFixedSize(24, 24)
        close_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        close_btn.setStyleSheet("color: rgba(255,255,255,0.6); background: transparent; border: none; font-size: 14px;")
        close_btn.clicked.connect(self.hide_sheet)
        header_row.addWidget(close_btn)

        layout.addLayout(header_row)

        # Quality Row
        q_row = QHBoxLayout()
        q_lbl = QLabel("Quality:")
        q_lbl.setFixedWidth(100)
        q_lbl.setStyleSheet("color: rgba(255,255,255,0.7); font-size: 13px;")
        q_row.addWidget(q_lbl)

        self.q_btn_box = QHBoxLayout()
        self.q_btn_box.setSpacing(8)
        q_row.addLayout(self.q_btn_box)
        q_row.addStretch()
        layout.addLayout(q_row)

        # Speed Row
        speed_row = QHBoxLayout()
        s_lbl = QLabel("Speed:")
        s_lbl.setFixedWidth(100)
        s_lbl.setStyleSheet("color: rgba(255,255,255,0.7); font-size: 13px;")
        speed_row.addWidget(s_lbl)

        self.speed_buttons = {}
        for spd in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0]:
            btn = QPushButton(f"{spd}x")
            btn.setProperty("class", "chip")
            btn.setCheckable(True)
            if spd == 1.0:
                btn.setChecked(True)
            btn.setCursor(Qt.CursorShape.PointingHandCursor)
            btn.clicked.connect(lambda _, val=spd: self._on_speed_clicked(val))
            speed_row.addWidget(btn)
            self.speed_buttons[spd] = btn
        speed_row.addStretch()
        layout.addLayout(speed_row)

        # Subtitle & Audio Dropdowns Row
        tracks_row = QHBoxLayout()
        sub_lbl = QLabel("Subtitles:")
        sub_lbl.setStyleSheet("color: rgba(255,255,255,0.7); font-size: 13px;")
        tracks_row.addWidget(sub_lbl)

        self.sub_combo = QComboBox(self)
        self.sub_combo.currentIndexChanged.connect(lambda idx: self.sub_track_changed.emit(idx))
        tracks_row.addWidget(self.sub_combo)

        tracks_row.addSpacing(24)

        audio_lbl = QLabel("Audio:")
        audio_lbl.setStyleSheet("color: rgba(255,255,255,0.7); font-size: 13px;")
        tracks_row.addWidget(audio_lbl)

        self.audio_combo = QComboBox(self)
        self.audio_combo.currentIndexChanged.connect(lambda idx: self.audio_track_changed.emit(idx))
        tracks_row.addWidget(self.audio_combo)
        tracks_row.addStretch()
        layout.addLayout(tracks_row)

        # Subtitle Delay Offset Row
        delay_row = QHBoxLayout()
        d_lbl = QLabel("Sub Delay:")
        d_lbl.setFixedWidth(100)
        d_lbl.setStyleSheet("color: rgba(255,255,255,0.7); font-size: 13px;")
        delay_row.addWidget(d_lbl)

        for delay_val, label in [(-1.0, "-1.0s"), (-0.5, "-0.5s"), (0.0, "0s"), (0.5, "+0.5s"), (1.0, "+1.0s")]:
            d_btn = QPushButton(label)
            d_btn.setProperty("class", "chip")
            d_btn.setCursor(Qt.CursorShape.PointingHandCursor)
            d_btn.clicked.connect(lambda _, d=delay_val: self.sub_delay_changed.emit(d))
            delay_row.addWidget(d_btn)
        delay_row.addStretch()
        layout.addLayout(delay_row)

    def populate_qualities(self, streams: list):
        while self.q_btn_box.count() > 0:
            child = self.q_btn_box.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        for stream in streams:
            q_text = stream.quality if hasattr(stream, 'quality') and stream.quality else "Auto"
            btn = QPushButton(q_text)
            btn.setProperty("class", "chip")
            btn.setCursor(Qt.CursorShape.PointingHandCursor)
            btn.clicked.connect(lambda _, st=stream: self.quality_changed.emit(st))
            self.q_btn_box.addWidget(btn)

    def populate_subtitles(self, tracks: list):
        self.sub_combo.blockSignals(True)
        self.sub_combo.clear()
        for t in tracks:
            self.sub_combo.addItem(t.get("label") or t.get("lang") or "Track", t.get("id"))
        self.sub_combo.blockSignals(False)

    def populate_audio(self, tracks: list):
        self.audio_combo.blockSignals(True)
        self.audio_combo.clear()
        for t in tracks:
            self.audio_combo.addItem(t.get("label") or t.get("lang") or "Audio", t.get("id"))
        self.audio_combo.blockSignals(False)

    def _on_speed_clicked(self, speed_val: float):
        for val, btn in self.speed_buttons.items():
            btn.setChecked(val == speed_val)
        self.speed_changed.emit(speed_val)

    def show_sheet(self, parent_rect: QRect):
        self.setGeometry(0, parent_rect.height() - 300, parent_rect.width(), 300)
        self.show()
        self.raise_()

    def hide_sheet(self):
        self.hide()
