import asyncio
from PyQt6.QtCore import QObject, pyqtSignal
import anilili_core
from utils.backend_bridge import BackendBridge
from utils.download_manager import DownloadManager

class BulkDownloadManager(QObject):
    bulk_progress = pyqtSignal(int, int)  # completed, total
    bulk_complete = pyqtSignal()
    bulk_cancelled = pyqtSignal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.dl_mgr = DownloadManager.instance()
        self._cancelled = False
        self._total = 0
        self._completed = 0
        self._completion_events = {}  # download_id -> asyncio.Event (Feedback #4)

        self.dl_mgr.download_complete.connect(self._on_download_finished)
        self.dl_mgr.download_error.connect(self._on_download_finished)

    def start_bulk(
        self,
        episodes: list,
        anilist_id: int,
        mal_id: int | None,
        series_title: str,
        series_cover: str | None,
        quality: str = "720p",
        category: str = "sub"
    ):
        self._cancelled = False
        self._total = len(episodes)
        self._completed = 0
        self._completion_events.clear()

        asyncio.create_task(
            self._run_bulk(episodes, anilist_id, mal_id, series_title, series_cover, quality, category)
        )

    async def _run_bulk(
        self,
        episodes: list,
        anilist_id: int,
        mal_id: int | None,
        series_title: str,
        series_cover: str | None,
        quality: str,
        category: str
    ):
        bridge = BackendBridge.instance()

        for i, ep in enumerate(episodes):
            if self._cancelled:
                self.bulk_cancelled.emit()
                return

            # Check storage before starting episode
            check = anilili_core.check_storage_sync(quality)
            if not check.ok:

                print(f"[BulkDownloadManager] Storage limit reached at episode {getattr(ep, 'number', i+1)}")
                break

            ep_num = getattr(ep, "number", float(i + 1))

            # Resolve episode stream source
            loop = asyncio.get_event_loop()
            future = loop.create_future()

            def _on_sources(success, streams, err):
                if not future.done():
                    future.set_result(streams if success else [])

            bridge.episode_sources_loaded.connect(lambda streams: _on_sources(True, streams, None))
            bridge.load_episode_sources(anilist_id, mal_id, ep_num, category)

            try:
                streams = await asyncio.wait_for(future, timeout=15.0)
            except Exception:
                streams = []

            stream_item = streams[0] if streams else None
            if stream_item:
                download_id = self.dl_mgr.start_download(
                    episode=ep,
                    stream_item=stream_item,
                    series_title=series_title,
                    series_cover=series_cover,
                    anilist_id=anilist_id,
                    quality=quality,
                    category=category
                )

                if download_id:
                    # Wait for completion before next (sequential download) via asyncio.Event (Feedback #4)
                    await self._wait_for_download(download_id)

            self._completed += 1
            self.bulk_progress.emit(self._completed, self._total)

            if i < len(episodes) - 1:
                await asyncio.sleep(1.2)  # 1.2s delay between downloads

        self.bulk_complete.emit()

    async def _wait_for_download(self, download_id: str):
        event = asyncio.Event()
        self._completion_events[download_id] = event
        try:
            await asyncio.wait_for(event.wait(), timeout=600.0)  # 10 minute timeout per download
        except asyncio.TimeoutError:
            pass
        finally:
            self._completion_events.pop(download_id, None)

    def _on_download_finished(self, download_id: str, *args):
        if download_id in self._completion_events:
            self._completion_events[download_id].set()

    def cancel(self):
        self._cancelled = True
        for event in list(self._completion_events.values()):
            event.set()
