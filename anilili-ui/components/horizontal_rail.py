from PyQt6.QtCore import pyqtSignal, Qt
from PyQt6.QtWidgets import QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QFrame, QScrollArea
from components.anime_card import AnimeCard

class HorizontalAnimeRail(QWidget):
    card_clicked = pyqtSignal(object)
    see_all_clicked = pyqtSignal(str)

    def __init__(self, title: str, category_key: str = "", parent=None):
        super().__init__(parent)
        self.category_key = category_key

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(10)

        # Header bar
        header = QHBoxLayout()
        header.setContentsMargins(0, 0, 0, 0)

        self.title_label = QLabel(title.upper())
        self.title_label.setStyleSheet("""
            QLabel {
                color: #ffffff;
                font-size: 15px;
                font-weight: bold;
                letter-spacing: 0.5px;
            }
        """)
        header.addWidget(self.title_label)
        header.addStretch()

        self.see_all_btn = QPushButton("See all →")
        self.see_all_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.see_all_btn.setStyleSheet("""
            QPushButton {
                color: #8979F2;
                font-size: 12px;
                font-weight: 600;
                background: transparent;
                border: none;
            }
            QPushButton:hover {
                color: #a99ef5;
            }
        """)
        self.see_all_btn.clicked.connect(lambda: self.see_all_clicked.emit(self.category_key))
        header.addWidget(self.see_all_btn)

        layout.addLayout(header)

        # Scroll Area for horizontal cards
        self.scroll_area = QScrollArea(self)


        self.scroll_area.setFixedHeight(240)
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        self.scroll_area.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")

        self.cards_container = QWidget()
        self.cards_container.setStyleSheet("background: transparent;")
        self.cards_layout = QHBoxLayout(self.cards_container)
        self.cards_layout.setContentsMargins(0, 0, 0, 0)
        self.cards_layout.setSpacing(12)
        self.cards_layout.setAlignment(Qt.AlignmentFlag.AlignLeft)

        self.scroll_area.setWidget(self.cards_container)
        layout.addWidget(self.scroll_area)

    def set_items(self, media_list):
        # Clear existing cards
        while self.cards_layout.count() > 0:
            child = self.cards_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()

        for media in media_list:
            card = AnimeCard(media=media, parent=self.cards_container)
            card.card_clicked.connect(self.card_clicked.emit)
            self.cards_layout.addWidget(card)

    def wheelEvent(self, event):
        # Translate vertical scroll wheel into horizontal scroll
        delta = event.angleDelta().y()
        self.scroll_area.horizontalScrollBar().setValue(
            self.scroll_area.horizontalScrollBar().value() - delta
        )
        event.accept()
