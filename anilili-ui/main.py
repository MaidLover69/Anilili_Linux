import sys
import os
import locale
try:
    locale.setlocale(locale.LC_NUMERIC, 'C')
except Exception:
    pass
from pathlib import Path
import asyncio

# Ensure project root is in sys.path
PROJECT_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from PyQt6.QtCore import Qt, QObject, QEvent
from PyQt6.QtWidgets import QApplication, QMainWindow, QWidget, QHBoxLayout, QStackedWidget
import qasync

from components.sidebar_nav import SidebarNav
from screens.home import HomeScreen
from screens.discover import DiscoverScreen
from screens.detail import DetailScreen
from screens.watch import WatchScreen
from screens.library import LibraryScreen
from screens.schedule import ScheduleScreen
from screens.settings import SettingsScreen
from utils.notification_manager import NotificationManager

class GlobalWheelFilter(QObject):
    def eventFilter(self, obj, event):
        if event.type() == QEvent.Type.Wheel:
            popup = QApplication.activePopupWidget()
            if popup:
                popup.close()

            scroll_area = obj
            while scroll_area and not hasattr(scroll_area, 'verticalScrollBar'):
                scroll_area = scroll_area.parent()
            
            if scroll_area:
                v_bar = scroll_area.verticalScrollBar()
                h_bar = scroll_area.horizontalScrollBar()
                
                if (not v_bar.isVisible() or scroll_area.verticalScrollBarPolicy() == Qt.ScrollBarPolicy.ScrollBarAlwaysOff) and h_bar.isVisible():
                    delta = event.angleDelta().y() or event.angleDelta().x()
                    if delta != 0:
                        h_bar.setValue(h_bar.value() - delta)
                        event.accept()
                        return True
                elif v_bar.isVisible():
                    delta = event.angleDelta().y()
                    if delta != 0:
                        v_bar.setValue(v_bar.value() - (delta // 2))
                        event.accept()
                        return True
        return super().eventFilter(obj, event)


class MainWindow(QMainWindow):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Anilili Linux Desktop")
        self.resize(1280, 780)
        self.setMinimumSize(900, 600)

        self._previous_window_state = Qt.WindowState.WindowNoState

        # Central Widget & Outer Layout
        central = QWidget(self)
        central.setObjectName("central_widget")
        self.setCentralWidget(central)

        main_layout = QHBoxLayout(central)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.setSpacing(0)

        # Sidebar Navigation
        self.sidebar = SidebarNav(central)
        self.sidebar.page_changed.connect(self._on_page_changed)
        main_layout.addWidget(self.sidebar)

        # QStackedWidget Page Router
        self.stacked_widget = QStackedWidget(central)
        main_layout.addWidget(self.stacked_widget)

        # Create Screens
        self.home_screen = HomeScreen(self.stacked_widget)
        self.home_screen.media_selected.connect(self._on_media_selected)
        self.home_screen.navigate_discover.connect(self._on_navigate_discover)
        self.stacked_widget.addWidget(self.home_screen)

        self.discover_screen = DiscoverScreen(self.stacked_widget)
        self.discover_screen.media_selected.connect(self._on_media_selected)
        self.stacked_widget.addWidget(self.discover_screen)

        self.schedule_screen = ScheduleScreen(self.stacked_widget)
        self.schedule_screen.media_selected.connect(self._on_media_selected)
        self.stacked_widget.addWidget(self.schedule_screen)

        self.detail_screen = DetailScreen(self.stacked_widget)
        self.detail_screen.back_clicked.connect(self._on_back_from_detail)
        self.detail_screen.navigate_discover.connect(self._on_navigate_discover_from_detail)
        self.detail_screen.play_episode_requested.connect(self._on_play_episode_requested)
        self.stacked_widget.addWidget(self.detail_screen)

        self.watch_screen = WatchScreen(self.stacked_widget)
        self.watch_screen.exit_requested.connect(self._on_exit_watch_requested)
        self.stacked_widget.addWidget(self.watch_screen)

        self.library_screen = LibraryScreen(self.stacked_widget)
        self.library_screen.media_selected.connect(self._on_media_selected)
        self.stacked_widget.addWidget(self.library_screen)

        self.settings_screen = SettingsScreen(self.stacked_widget)
        self.stacked_widget.addWidget(self.settings_screen)

        # Route mapping
        self.routes = {
            "home": self.home_screen,
            "discover": self.discover_screen,
            "schedule": self.schedule_screen,
            "detail": self.detail_screen,
            "watch": self.watch_screen,
            "library": self.library_screen,
            "settings": self.settings_screen,
        }

        # Apply QSS Stylesheet
        self._load_stylesheet()

        # Check for updates on startup if enabled in settings
        self._check_updates_on_startup()

    def _check_updates_on_startup(self):
        from settings.store import SettingsStore
        from utils.backend_bridge import BackendBridge
        from components.update_dialog import UpdateDialog

        if SettingsStore.instance().get("update_check_on_launch", True):
            def _on_update(success, info, err):
                if success and info:
                    dialog = UpdateDialog(info, self)
                    dialog.exec()

            BackendBridge.instance().check_for_update(_on_update)

    def _load_stylesheet(self):

        qss_path = Path(__file__).parent / "theme" / "stylesheet.qss"
        if qss_path.exists():
            with open(qss_path, "r", encoding="utf-8") as f:
                self.setStyleSheet(f.read())

    def _on_page_changed(self, route: str):
        if route in self.routes:
            self.stacked_widget.setCurrentWidget(self.routes[route])

    def _on_media_selected(self, anilist_id: int):
        self.detail_screen.load_anime(anilist_id)
        self.stacked_widget.setCurrentWidget(self.detail_screen)

    def _on_navigate_discover(self, category_key: str):
        self.sidebar.set_active("discover")
        self.stacked_widget.setCurrentWidget(self.discover_screen)

    def _on_navigate_discover_from_detail(self, filter_dict: dict):
        self.sidebar.set_active("discover")
        if "genre" in filter_dict:
            self.discover_screen.set_genre_filter(filter_dict["genre"])
        self.stacked_widget.setCurrentWidget(self.discover_screen)

    def _on_play_episode_requested(self, play_info: dict | object):
        self._previous_window_state = self.windowState()

        # Hide sidebar before going fullscreen
        self.sidebar.setVisible(False)

        # Enter fullscreen / windowed-fullscreen
        try:
            self.showFullScreen()
        except Exception:
            self.showMaximized()

        self.stacked_widget.setCurrentWidget(self.watch_screen)

        # Extract mal_id & title_romaji from current details (dict, not object)
        details = getattr(self.detail_screen, '_current_details', {}) or {}
        if isinstance(details, dict):
            mal_id = details.get("idMal")
            title_obj = details.get("title", {}) if isinstance(details.get("title"), dict) else {}
            title_romaji = title_obj.get("romaji") or title_obj.get("userPreferred") or title_obj.get("english") or ""
            anime_title = title_obj.get("english") or title_obj.get("userPreferred") or title_obj.get("romaji") or self.detail_screen.title_label.text()
        else:
            mal_id = getattr(details, "id_mal", None)
            title_romaji = getattr(details, "title_str", "")
            anime_title = getattr(details, "title_str", "")

        if isinstance(play_info, dict):
            ep_num = play_info.get("episode_number", 1.0)
            ep_title = play_info.get("episode_title", "")
            cat = play_info.get("category", "sub")
        else:
            ep_num = getattr(play_info, 'number', 1.0)
            ep_title = getattr(play_info, 'title', "") or ""
            cat = "sub"

        print(f"[Main] _on_play_episode_requested: ep={ep_num}, mal_id={mal_id}, title_romaji='{title_romaji}'")

        self.watch_screen.load_episode(
            anilist_id=getattr(self.detail_screen, '_anilist_id', 0),
            mal_id=mal_id,
            episode_number=ep_num,
            category=cat,
            anime_title=anime_title,
            episode_title=ep_title,
            title_romaji=title_romaji
        )

    def _on_exit_watch_requested(self):
        # Restore sidebar & window state
        self.sidebar.setVisible(True)
        if self.isFullScreen():
            self.showNormal()

        self.stacked_widget.setCurrentWidget(self.detail_screen)

    def _on_back_from_detail(self):
        self.sidebar.set_active("home")
        self.stacked_widget.setCurrentWidget(self.home_screen)

    def closeEvent(self, event):
        from utils.aria2_manager import Aria2Manager
        Aria2Manager.instance().stop_aria2c()
        NotificationManager.instance().stop()
        super().closeEvent(event)


def main():
    from utils.aria2_manager import Aria2Manager
    Aria2Manager.instance().start_aria2c()

    app = QApplication(sys.argv)
    wheel_filter = GlobalWheelFilter(app)
    app.installEventFilter(wheel_filter)

    loop = qasync.QEventLoop(app)
    asyncio.set_event_loop(loop)

    NotificationManager.instance().start()

    window = MainWindow()
    window.show()

    with loop:
        loop.run_forever()


if __name__ == "__main__":
    main()

