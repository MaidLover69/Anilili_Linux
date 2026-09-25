from PyQt6.QtCore import pyqtSignal, Qt
from PyQt6.QtWidgets import QWidget, QVBoxLayout, QFrame, QScrollArea
from components.hero_banner import HeroBanner
from components.horizontal_rail import HorizontalAnimeRail
from viewmodels.home_vm import HomeViewModel

class HomeScreen(QWidget):
    media_selected = pyqtSignal(int)  # Emits anilist_id integer
    navigate_discover = pyqtSignal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.vm = HomeViewModel(self)
        self.vm.state_changed.connect(self._on_state_changed)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # Scroll Area
        self.scroll_area = QScrollArea(self)

        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        self.scroll_area.setStyleSheet("background: transparent;")


        self.container = QWidget()
        self.container.setStyleSheet("background: transparent;")
        self.content_layout = QVBoxLayout(self.container)
        self.content_layout.setContentsMargins(0, 0, 0, 32)
        self.content_layout.setSpacing(24)

        # Hero Banner
        self.hero_banner = HeroBanner(self.container)
        self.hero_banner.watch_clicked.connect(self._on_card_clicked)
        self.hero_banner.details_clicked.connect(self._on_card_clicked)
        self.content_layout.addWidget(self.hero_banner)

        # Continue Watching Rail
        self.continue_rail = HorizontalAnimeRail("Continue Watching", "continue_watching", self.container)
        self.continue_rail.card_clicked.connect(self._on_card_clicked)
        self.continue_rail.setVisible(False) # Hidden by default if empty
        self.content_layout.addWidget(self.continue_rail)

        # Trending Rail
        self.trending_rail = HorizontalAnimeRail("Trending Now", "trending", self.container)
        self.trending_rail.card_clicked.connect(self._on_card_clicked)
        self.trending_rail.see_all_clicked.connect(self._on_see_all)
        self.content_layout.addWidget(self.trending_rail)

        # Popular Rail
        self.popular_rail = HorizontalAnimeRail("All Time Popular", "popular", self.container)
        self.popular_rail.card_clicked.connect(self._on_card_clicked)
        self.popular_rail.see_all_clicked.connect(self._on_see_all)
        self.content_layout.addWidget(self.popular_rail)

        # Top Rated Rail
        self.top_rated_rail = HorizontalAnimeRail("Top Rated", "top_rated", self.container)
        self.top_rated_rail.card_clicked.connect(self._on_card_clicked)
        self.top_rated_rail.see_all_clicked.connect(self._on_see_all)
        self.content_layout.addWidget(self.top_rated_rail)

        self.scroll_area.setWidget(self.container)
        layout.addWidget(self.scroll_area)

        # Initial data load
        self.vm.load()

    def _on_state_changed(self, state: dict):
        if state["loading"]:
            return

        data = state["data"]

        # Cap hero banner items to top 5 trending items
        trending_items = data.get("trending", [])
        hero_items = trending_items[:5]
        self.hero_banner.set_items(hero_items)

        # Continue Watching Rail (Hide when empty)
        continue_items = data.get("continue_watching", [])
        if continue_items:
            self.continue_rail.set_items(continue_items)
            self.continue_rail.setVisible(True)
        else:
            self.continue_rail.setVisible(False)

        # Rails
        self.trending_rail.set_items(trending_items)
        self.popular_rail.set_items(data.get("popular", []))
        self.top_rated_rail.set_items(data.get("top_rated", []))

    def _on_card_clicked(self, media):
        anilist_id = media.id if hasattr(media, 'id') else getattr(media, 'anilist_id', 0)
        if anilist_id:
            self.media_selected.emit(int(anilist_id))

    def _on_see_all(self, category_key: str):
        self.navigate_discover.emit(category_key)
