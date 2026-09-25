from PyQt6.QtCore import Qt, QUrl
from PyQt6.QtGui import QDesktopServices
from PyQt6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QTextEdit, QPushButton, QFrame
)

class UpdateDialog(QDialog):
    def __init__(self, update_info, parent=None):
        super().__init__(parent)
        self.update_info = update_info
        self.setWindowTitle("Update Available")
        self.setFixedSize(520, 420)
        self.setStyleSheet("""
            QDialog {
                background-color: #141416;
                color: #ffffff;
            }
            QLabel {
                color: #ffffff;
            }
            QTextEdit {
                background-color: #1c1c1f;
                color: rgba(255, 255, 255, 0.9);
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 8px;
                padding: 10px;
                font-size: 13px;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        # Header Title & Version Tag
        header_layout = QVBoxLayout()
        header_layout.setSpacing(4)

        tag_name = getattr(update_info, 'version', '1.1.0')
        title_label = QLabel(f"🚀 Anilili v{tag_name} Available!")
        title_label.setStyleSheet("font-size: 20px; font-weight: bold; color: #8979F2;")
        header_layout.addWidget(title_label)

        sub_label = QLabel("A new version of Anilili Linux is ready to download.")
        sub_label.setStyleSheet("font-size: 13px; color: rgba(255, 255, 255, 0.6);")
        header_layout.addWidget(sub_label)

        layout.addLayout(header_layout)

        # Changelog Text Area
        changelog_label = QLabel("Release Notes & Changelog:")
        changelog_label.setStyleSheet("font-size: 13px; font-weight: bold;")
        layout.addWidget(changelog_label)

        self.changelog_text = QTextEdit(self)
        self.changelog_text.setReadOnly(True)
        changelog_body = getattr(update_info, 'changelog', 'No release notes provided.')
        self.changelog_text.setPlainText(changelog_body)
        layout.addWidget(self.changelog_text, stretch=1)

        # Action Buttons Row
        btn_row = QHBoxLayout()
        btn_row.setSpacing(12)

        self.dismiss_btn = QPushButton("Dismiss", self)
        self.dismiss_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.dismiss_btn.setStyleSheet("""
            QPushButton {
                background-color: rgba(255, 255, 255, 0.08);
                color: #ffffff;
                border-radius: 6px;
                padding: 8px 16px;
                font-weight: 600;
            }
            QPushButton:hover {
                background-color: rgba(255, 255, 255, 0.15);
            }
        """)
        self.dismiss_btn.clicked.connect(self.reject)
        btn_row.addWidget(self.dismiss_btn)

        btn_row.addStretch()

        self.github_btn = QPushButton("🌐 View on GitHub", self)
        self.github_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.github_btn.setStyleSheet("""
            QPushButton {
                background-color: #8979F2;
                color: #ffffff;
                border-radius: 6px;
                padding: 8px 20px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #9b8df4;
            }
        """)
        self.github_btn.clicked.connect(self._on_github_clicked)
        btn_row.addWidget(self.github_btn)

        layout.addLayout(btn_row)

    def _on_github_clicked(self):
        url = getattr(self.update_info, 'release_url', 'https://github.com/kompoti121/Anilili/releases')
        QDesktopServices.openUrl(QUrl(url))
        self.accept()
