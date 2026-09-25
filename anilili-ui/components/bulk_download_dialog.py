from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QRadioButton, QButtonGroup, QProgressBar, QFrame
)
import anilili_core
from utils.bulk_download_manager import BulkDownloadManager

class BulkDownloadDialog(QDialog):
    def __init__(self, episodes: list, anilist_id: int, mal_id: int | None, series_title: str, series_cover: str | None, parent=None):
        super().__init__(parent)
        self.episodes = episodes
        self.anilist_id = anilist_id
        self.mal_id = mal_id
        self.series_title = series_title
        self.series_cover = series_cover
        self.bulk_mgr = BulkDownloadManager(self)

        self.setWindowTitle("Download Episodes")
        self.resize(420, 380)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        title_lbl = QLabel(f"Download — {series_title}")
        title_lbl.setStyleSheet("color: #ffffff; font-size: 16px; font-weight: bold;")
        layout.addWidget(title_lbl)

        # 1. Episode Count Group
        count_lbl = QLabel("SELECT EPISODE COUNT:")
        count_lbl.setStyleSheet("color: rgba(255,255,255,0.6); font-size: 11px; font-weight: bold;")
        layout.addWidget(count_lbl)

        count_row = QHBoxLayout()
        self.count_group = QButtonGroup(self)

        total_eps = len(episodes)
        counts = [5, 10, 25]
        for c in counts:
            if c <= total_eps:
                rb = QRadioButton(f"{c} Episodes", self)
                rb.setStyleSheet("color: #ffffff; font-size: 12px;")
                self.count_group.addButton(rb, c)
                count_row.addWidget(rb)

        rb_all = QRadioButton(f"All ({total_eps})", self)
        rb_all.setChecked(True)
        rb_all.setStyleSheet("color: #ffffff; font-size: 12px;")
        self.count_group.addButton(rb_all, total_eps)
        count_row.addWidget(rb_all)

        layout.addLayout(count_row)

        # 2. Quality Selection Group
        quality_lbl = QLabel("PREFERRED QUALITY:")
        quality_lbl.setStyleSheet("color: rgba(255,255,255,0.6); font-size: 11px; font-weight: bold;")
        layout.addWidget(quality_lbl)

        quality_row = QHBoxLayout()
        self.quality_group = QButtonGroup(self)

        qualities = [("1080p", "1080p"), ("720p", "720p"), ("480p", "480p")]
        for idx, (label, val) in enumerate(qualities):
            rb = QRadioButton(label, self)
            if idx == 0:
                rb.setChecked(True)
            rb.setStyleSheet("color: #ffffff; font-size: 12px;")
            self.quality_group.addButton(rb)
            rb.setProperty("quality_val", val)
            quality_row.addWidget(rb)

        layout.addLayout(quality_row)

        # 3. Storage Estimate Label
        self.storage_lbl = QLabel("Calculating storage...", self)
        self.storage_lbl.setStyleSheet("color: #8979F2; font-size: 12px;")
        layout.addWidget(self.storage_lbl)

        # 4. Progress Bar & Status (Hidden until download starts)
        self.progress_bar = QProgressBar(self)
        self.progress_bar.setVisible(False)
        self.progress_bar.setStyleSheet("""
            QProgressBar {
                background-color: #1a1a1d;
                border-radius: 4px;
                text-align: center;
                color: white;
            }
            QProgressBar::chunk {
                background-color: #8979F2;
                border-radius: 4px;
            }
        """)
        layout.addWidget(self.progress_bar)

        # 5. Buttons
        btn_row = QHBoxLayout()
        btn_row.addStretch()

        self.cancel_btn = QPushButton("Cancel", self)
        self.cancel_btn.setProperty("class", "secondary")
        self.cancel_btn.clicked.connect(self._on_cancel)
        btn_row.addWidget(self.cancel_btn)

        self.start_btn = QPushButton("Start Download", self)
        self.start_btn.setProperty("class", "primary")
        self.start_btn.clicked.connect(self._on_start)
        btn_row.addWidget(self.start_btn)

        layout.addLayout(btn_row)

        self.bulk_mgr.bulk_progress.connect(self._on_progress)
        self.bulk_mgr.bulk_complete.connect(self._on_complete)
        self.bulk_mgr.bulk_cancelled.connect(self._on_cancelled)

        self._update_storage_estimate()

    def _get_selected_quality(self) -> str:

        button = self.quality_group.checkedButton()
        if button:
            return button.property("quality_val") or "720p"
        return "720p"

    def _update_storage_estimate(self):
        quality = self._get_selected_quality()
        count = self.count_group.checkedId()
        if count <= 0:
            count = len(self.episodes)

        check = anilili_core.check_storage_sync(quality)
        needed_gb = (check.needed_bytes * count) / (1024**3)

        free_gb = check.free_bytes / (1024**3)

        self.storage_lbl.setText(f"~{needed_gb:.1f} GB required, {free_gb:.1f} GB available")

    def _on_start(self):
        count = self.count_group.checkedId()
        if count <= 0:
            count = len(self.episodes)
        selected_eps = self.episodes[:count]

        quality = self._get_selected_quality()

        self.start_btn.setEnabled(False)
        self.progress_bar.setVisible(True)
        self.progress_bar.setMaximum(len(selected_eps))
        self.progress_bar.setValue(0)

        self.bulk_mgr.start_bulk(
            episodes=selected_eps,
            anilist_id=self.anilist_id,
            mal_id=self.mal_id,
            series_title=self.series_title,
            series_cover=self.series_cover,
            quality=quality
        )

    def _on_progress(self, completed: int, total: int):
        self.progress_bar.setValue(completed)
        self.storage_lbl.setText(f"Downloading episode {completed} of {total}...")

    def _on_complete(self):
        self.accept()

    def _on_cancelled(self):
        self.reject()

    def _on_cancel(self):
        self.bulk_mgr.cancel()
        self.reject()
