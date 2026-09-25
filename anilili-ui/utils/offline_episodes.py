import os
import glob
import asyncio
from utils.backend_bridge import BackendBridge

class OfflineEpisodeManager:
    @staticmethod
    async def get_offline_episodes(anilist_id: int) -> list:
        bridge = BackendBridge.instance()
        loop = asyncio.get_event_loop()
        future = loop.create_future()

        def _on_downloads(success, records, err):
            if not future.done():
                if success:
                    # Filter records that are SAVED and file exists
                    valid = [r for r in records if r.status == "SAVED" and r.file_path and os.path.exists(r.file_path)]
                    future.set_result(valid)
                else:
                    future.set_result([])

        bridge.downloads_loaded.connect(lambda records: _on_downloads(True, records, None))
        bridge.load_downloads(anilist_id)

        try:
            return await asyncio.wait_for(future, timeout=5.0)
        except Exception:
            return []

    @staticmethod
    def find_sidecars(file_path: str) -> list[str]:
        if not file_path:
            return []
        base_path, _ = os.path.splitext(file_path)
        sidecars = []
        for ext in ["vtt", "srt", "ass"]:
            sidecars.extend(glob.glob(f"{base_path}*.{ext}"))
        return sidecars

    @classmethod
    def build_player_args(cls, record) -> dict:
        file_path = record.file_path if hasattr(record, "file_path") else str(record)
        sidecars = cls.find_sidecars(file_path)
        return {
            "url": file_path,
            "subtitles": sidecars
        }
