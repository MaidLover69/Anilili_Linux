import time
from datetime import datetime, timedelta
from PyQt6.QtCore import pyqtSignal, Qt, QTimer
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QFrame,
    QStackedWidget, QScrollArea
)
from components.shimmer_loader import ShimmerImageWidget
from viewmodels.schedule_vm import ScheduleViewModel

class AiringCard(QFrame):
    card_clicked = pyqtSignal(int)       # Emits media_id
    notify_toggled = pyqtSignal(object)   # Emits AiringEntry

    def __init__(self, entry, is_notified=False, is_filler=False, parent=None):
        super().__init__(parent)
        self.entry = entry
        self.is_notified = is_notified
        self.setFixedHeight(76)
        self.setCursor(Qt.CursorShape.PointingHandCursor)

        self.setStyleSheet("""
            AiringCard {
                background-color: #1a1a1d;
                border: 1px solid rgba(255, 255, 255, 0.08);
                border-radius: 8px;
            }
            AiringCard:hover {
                background-color: #222226;
                border-color: rgba(137, 121, 242, 0.4);
            }
        """)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 8, 12, 8)
        layout.setSpacing(16)

        # 1. Local Airing Time (HH:MM)
        airing_at = getattr(entry, 'airing_at', 0)
        dt = datetime.fromtimestamp(airing_at)
        time_str = dt.strftime("%H:%M")

        self.time_label = QLabel(time_str)
        self.time_label.setFixedWidth(52)
        self.time_label.setStyleSheet("color: #ffffff; font-size: 15px; font-weight: bold;")
        layout.addWidget(self.time_label)

        # 2. Cover Thumbnail (46x60px)
        self.thumb_widget = ShimmerImageWidget(width=46, height=60, border_radius=6, parent=self)
        cover_url = getattr(entry, 'cover_image', None)
        if cover_url:
            self.thumb_widget.load_url(cover_url)
        layout.addWidget(self.thumb_widget)

        # 3. Info Column (Title + Episode + Countdown Pill)
        info_col = QVBoxLayout()
        info_col.setSpacing(4)

        media_title = getattr(entry, 'media_title', 'Untitled Anime')
        self.title_label = QLabel(media_title)
        self.title_label.setWordWrap(False)
        self.title_label.setStyleSheet("color: #ffffff; font-size: 14px; font-weight: bold;")
        info_col.addWidget(self.title_label)

        meta_row = QHBoxLayout()
        meta_row.setSpacing(8)

        ep_num = getattr(entry, 'episode', 1)
        self.ep_label = QLabel(f"Episode {ep_num}")
        self.ep_label.setStyleSheet("color: rgba(255, 255, 255, 0.6); font-size: 12px;")
        meta_row.addWidget(self.ep_label)

        # Countdown Pill
        self.pill_label = QLabel()
        self.update_countdown()
        meta_row.addWidget(self.pill_label)
        meta_row.addStretch()

        info_col.addLayout(meta_row)
        layout.addLayout(info_col, stretch=1)

        # 4. Filler Badge (if applicable)
        if is_filler:
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

        # 5. Notification Bell Toggle Button (32x32px)
        self.bell_btn = QPushButton("🔔" if is_notified else "🔕")
        self.bell_btn.setFixedSize(32, 32)
        self.bell_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.bell_btn.setToolTip("Toggle Desktop Notification")
        self._update_bell_style()
        self.bell_btn.clicked.connect(self._on_bell_clicked)
        layout.addWidget(self.bell_btn)

    def _update_bell_style(self):
        if self.is_notified:
            self.bell_btn.setText("🔔")
            self.bell_btn.setStyleSheet("""
                QPushButton {
                    background-color: #8979F2;
                    color: #ffffff;
                    border-radius: 16px;
                    font-size: 13px;
                    border: none;
                }
                QPushButton:hover {
                    background-color: #9b8df4;
                }
            """)
        else:
            self.bell_btn.setText("🔕")
            self.bell_btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(255, 255, 255, 0.06);
                    color: rgba(255, 255, 255, 0.5);
                    border-radius: 16px;
                    font-size: 13px;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                }
                QPushButton:hover {
                    background-color: rgba(255, 255, 255, 0.12);
                    color: #ffffff;
                }
            """)

    def _on_bell_clicked(self):

        self.is_notified = not self.is_notified
        self._update_bell_style()
        self.notify_toggled.emit(self.entry)

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            media_id = getattr(self.entry, 'media_id', 0)
            if media_id:
                self.card_clicked.emit(int(media_id))
        super().mousePressEvent(event)

    def update_countdown(self):
        airing_at = getattr(self.entry, 'airing_at', 0)
        now_ts = int(time.time())
        diff = airing_at - now_ts

        if diff > 0:
            hours = diff // 3600
            mins = (diff % 3600) // 60
            if hours > 0:
                text = f"Airing in {hours}h {mins}m"
            else:
                text = f"Airing in {mins}m"
            self.pill_label.setText(text)
            self.pill_label.setStyleSheet("""
                QLabel {
                    background-color: rgba(137, 121, 242, 0.2);
                    color: #8979F2;
                    font-size: 11px;
                    font-weight: bold;
                    padding: 2px 8px;
                    border-radius: 10px;
                }
            """)
        else:
            past_mins = abs(diff) // 60
            if past_mins < 60:
                text = f"Aired {past_mins}m ago"
            else:
                past_hours = past_mins // 60
                text = f"Aired {past_hours}h ago"
            self.pill_label.setText(text)
            self.pill_label.setStyleSheet("""
                QLabel {
                    background-color: rgba(255, 255, 255, 0.08);
                    color: rgba(255, 255, 255, 0.6);
                    font-size: 11px;
                    padding: 2px 8px;
                    border-radius: 10px;
                }
            """)


class ScheduleScreen(QWidget):
    media_selected = pyqtSignal(int)  # Emits anilist_id

    def __init__(self, parent=None):
        super().__init__(parent)
        self.vm = ScheduleViewModel(self)
        self.vm.state_changed.connect(self._on_state_changed)

        self._active_day_offset = 0
        self._cards = []

        layout = QVBoxLayout(self)
        layout.setContentsMargins(32, 24, 32, 32)
        layout.setSpacing(20)

        # Header Title
        header_title = QLabel("Airing Schedule")
        header_title.setStyleSheet("color: #ffffff; font-size: 24px; font-weight: bold;")
        layout.addWidget(header_title)

        # 7-Day Navigation Tab Bar
        self.tab_layout = QHBoxLayout()
        self.tab_layout.setSpacing(8)
        self.tab_buttons = {}

        for offset in range(-3, 4):
            btn = QPushButton()
            btn.setFixedHeight(40)
            btn.setCursor(Qt.CursorShape.PointingHandCursor)
            btn.clicked.connect(lambda checked, o=offset: self._on_tab_click(o))
            self.tab_layout.addWidget(btn)
            self.tab_buttons[offset] = btn

        self._render_tab_labels()
        layout.addLayout(self.tab_layout)

        # QStackedWidget for Content States (0: Loading, 1: Empty/Error, 2: Content)
        self.stacked_widget = QStackedWidget(self)

        # State 0: Loading
        loading_widget = QWidget()
        loading_layout = QVBoxLayout(loading_widget)
        loading_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        loading_label = QLabel("Loading schedule...")
        loading_label.setStyleSheet("color: rgba(255, 255, 255, 0.5); font-size: 14px;")
        loading_layout.addWidget(loading_label)
        self.stacked_widget.addWidget(loading_widget)

        # State 1: Empty / Error
        self.empty_widget = QWidget()
        empty_layout = QVBoxLayout(self.empty_widget)
        empty_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.empty_label = QLabel("No airing anime scheduled for this date.")
        self.empty_label.setStyleSheet("color: rgba(255, 255, 255, 0.5); font-size: 14px;")
        empty_layout.addWidget(self.empty_label)
        self.stacked_widget.addWidget(self.empty_widget)

        # State 2: Content Scroll View
        self.scroll_area = QScrollArea(self)

        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")

        self.cards_container = QWidget()
        self.cards_container.setStyleSheet("background: transparent;")
        self.cards_layout = QVBoxLayout(self.cards_container)
        self.cards_layout.setContentsMargins(0, 0, 0, 0)
        self.cards_layout.setSpacing(10)
        self.cards_layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        self.scroll_area.setWidget(self.cards_container)
        self.stacked_widget.addWidget(self.scroll_area)

        layout.addWidget(self.stacked_widget)

        # 60s Live Countdown Update Timer
        self.countdown_timer = QTimer(self)
        self.countdown_timer.setInterval(60000)
        self.countdown_timer.timeout.connect(self._update_all_countdowns)
        self.countdown_timer.start()

        # Initial load for Today (offset 0)
        self.vm.load_day(0)

    def _render_tab_labels(self):
        now = datetime.now()
        for offset, btn in self.tab_buttons.items():
            target_date = now + timedelta(days=offset)
            if offset == 0:
                day_str = "TODAY"
                date_str = target_date.strftime("%b %d")
                label_text = f"<b>{day_str}</b><br/><span style='font-size:11px;'>{date_str}</span>"
            else:
                day_str = target_date.strftime("%a")
                date_str = target_date.strftime("%b %d")
                label_text = f"{day_str}<br/><span style='font-size:11px;'>{date_str}</span>"

            btn.setText(f"{day_str} · {date_str}")
            self._style_tab_button(btn, offset == self._active_day_offset, offset == 0)

    def _style_tab_button(self, btn, is_active, is_today):
        if is_active:
            btn.setStyleSheet("""
                QPushButton {
                    background-color: #8979F2;
                    color: #ffffff;
                    font-weight: bold;
                    font-size: 13px;
                    border-radius: 8px;
                    border: none;
                }
            """)
        elif is_today:
            btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(137, 121, 242, 0.15);
                    color: #8979F2;
                    font-weight: bold;
                    font-size: 13px;
                    border-radius: 8px;
                    border: 1px solid rgba(137, 121, 242, 0.4);
                }
                QPushButton:hover {
                    background-color: rgba(137, 121, 242, 0.25);
                }
            """)
        else:
            btn.setStyleSheet("""
                QPushButton {
                    background-color: rgba(255, 255, 255, 0.05);
                    color: rgba(255, 255, 255, 0.7);
                    font-size: 13px;
                    border-radius: 8px;
                    border: 1px solid rgba(255, 255, 255, 0.08);
                }
                QPushButton:hover {
                    background-color: rgba(255, 255, 255, 0.1);
                    color: #ffffff;
                }
            """)

    def _on_tab_click(self, offset: int):
        self._active_day_offset = offset
        for o, btn in self.tab_buttons.items():
            self._style_tab_button(btn, o == offset, o == 0)
        self.vm.load_day(offset)

    def _on_state_changed(self, state: dict):
        if state["loading"]:
            self.stacked_widget.setCurrentIndex(0)
            return

        if state["error"]:
            self.empty_label.setText(f"Error loading schedule: {state['error']}")
            self.stacked_widget.setCurrentIndex(1)
            return

        entries = state["entries"]
        preferences = state["preferences"]

        if not entries:
            self.empty_label.setText("No airing anime scheduled for this date.")
            self.stacked_widget.setCurrentIndex(1)
            return

        # Render entries
        while self.cards_layout.count() > 0:
            child = self.cards_layout.takeAt(0)
            if child.widget():
                child.widget().deleteLater()
        self._cards.clear()

        for entry in entries:
            media_id = getattr(entry, 'media_id', 0)
            is_notified = preferences.get(media_id, False)

            card = AiringCard(entry, is_notified=is_notified, parent=self.cards_container)
            card.card_clicked.connect(self.media_selected.emit)
            card.notify_toggled.connect(self.vm.toggle_notification)
            self.cards_layout.addWidget(card)
            self._cards.append(card)

        self.stacked_widget.setCurrentIndex(2)

    def _update_all_countdowns(self):
        for card in self._cards:
            try:
                card.update_countdown()
            except Exception:
                pass
