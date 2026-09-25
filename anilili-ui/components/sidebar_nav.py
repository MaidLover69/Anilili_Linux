from PyQt6.QtCore import pyqtSignal, Qt, QPropertyAnimation, QEasingCurve
from PyQt6.QtWidgets import QWidget, QVBoxLayout, QPushButton, QLabel, QFrame, QSpacerItem, QSizePolicy

class SidebarNav(QFrame):
    page_changed = pyqtSignal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("sidebar")
        self._expanded = True
        self.setFixedWidth(220)

        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(12, 16, 12, 16)
        self.layout.setSpacing(8)

        # Header / Logo
        self.logo_btn = QPushButton("  A  Anilili", self)
        self.logo_btn.setStyleSheet("""
            QPushButton {
                color: #8979F2;
                font-size: 18px;
                font-weight: bold;
                text-align: left;
                border: none;
                background: transparent;
            }
        """)
        self.logo_btn.clicked.connect(self.toggle_collapse)
        self.layout.addWidget(self.logo_btn)

        # Separator line
        line = QFrame(self)
        line.setFrameShape(QFrame.Shape.HLine)
        line.setStyleSheet("color: rgba(255,255,255,0.07);")
        self.layout.addWidget(line)

        # Navigation items
        self.items = {}
        self._add_nav_item("home", "⌂", "Home")
        self._add_nav_item("discover", "🔍", "Discover")
        self._add_nav_item("schedule", "📅", "Schedule")
        self._add_nav_item("library", "📚", "Library")

        # Spacer
        self.layout.addSpacerItem(QSpacerItem(20, 40, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding))

        # Bottom items
        self._add_nav_item("settings", "⚙", "Settings")

        # Set default active item
        self.set_active("home")

    def _add_nav_item(self, route: str, icon: str, label: str):
        btn = QPushButton(f"{icon}   {label}", self)
        btn.setProperty("class", "nav_item")
        btn.setProperty("route", route)
        btn.setProperty("icon_str", icon)
        btn.setProperty("label_str", label)
        btn.setCursor(Qt.CursorShape.PointingHandCursor)
        btn.clicked.connect(lambda: self._on_item_clicked(route))
        self.layout.addWidget(btn)
        self.items[route] = btn

    def _on_item_clicked(self, route: str):
        self.set_active(route)
        self.page_changed.emit(route)

    def set_active(self, active_route: str):
        for route, btn in self.items.items():
            is_active = (route == active_route)
            btn.setProperty("active", "true" if is_active else "false")
            btn.style().unpolish(btn)
            btn.style().polish(btn)

    def toggle_collapse(self):
        target_width = 64 if self._expanded else 220
        self._expanded = not self._expanded

        self.anim = QPropertyAnimation(self, b"minimumWidth")
        self.anim.setDuration(200)
        self.anim.setStartValue(self.width())
        self.anim.setEndValue(target_width)
        self.anim.setEasingCurve(QEasingCurve.Type.InOutQuad)

        self.anim_max = QPropertyAnimation(self, b"maximumWidth")
        self.anim_max.setDuration(200)
        self.anim_max.setStartValue(self.width())
        self.anim_max.setEndValue(target_width)
        self.anim_max.setEasingCurve(QEasingCurve.Type.InOutQuad)

        self.anim.start()
        self.anim_max.start()

        # Update button text formatting
        for btn in self.items.values():
            icon = btn.property("icon_str")
            label = btn.property("label_str")
            btn.setText(f"{icon}   {label}" if self._expanded else icon)

        self.logo_btn.setText("  A  Anilili" if self._expanded else " A")
