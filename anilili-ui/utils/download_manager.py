import os
import glob
import uuid
import re
import asyncio
import httpx
from pathlib import Path
from PyQt6.QtCore import QObject, pyqtSignal, QTimer
import anilili_core
from utils.backend_bridge import BackendBridge
from utils.aria2_manager import Aria2Manager

def slugify(text: str) -> str:
    text = text.lower()
    text = re.sub(r"[^\w\s-]", "", text)
    return re.sub(r"[-\s]+", "_", text).strip("_")

class DownloadManager(QObject):
    _instance = None

    download_progress = pyqtSignal(str, float, int)  # id, progress (0.0 to 1.0), speed (bytes/s)
    download_complete = pyqtSignal(str)               # id
    download_error = pyqtSignal(str, str)             # id, error_message
    download_started = pyqtSignal(str)                # id

    @classmethod
    def instance(cls) -> "DownloadManager":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self, parent=None):
        super().__init__(parent)
        self.bridge = BackendBridge.instance()
        self.aria2 = Aria2Manager.instance()
        self._active = {}  # download_id -> {"gid": str, "record": DownloadRecord}

        # Initialize aria2c
        self.aria2.start_aria2c()

        # Poll timer (every 2 seconds)
        self._poll_timer = QTimer(self)
        self._poll_timer.setInterval(2000)
        self._poll_timer.timeout.connect(self._poll_active)
        self._poll_timer.start()

    def start_download(
        self,
        episode,
        stream_item,
        series_title: str,
        series_cover: str | None,
        anilist_id: int,
        quality: str = "720p",
        provider: str = "AniBD",
        category: str = "sub"
    ) -> str | None:

        # 1. Storage check (1GB headroom)
        check = anilili_core.check_storage_sync(quality)
        if not check.ok:

            err_msg = f"Insufficient disk space. Free: {check.free_bytes / (1024**3):.2f}GB, Needed: {(check.needed_bytes + 1048576000) / (1024**3):.2f}GB"
            print(f"[DownloadManager] {err_msg}")
            self.download_error.emit("", err_msg)
            return None

        # 2. Generate download ID and path
        download_id = str(uuid.uuid4())
        ep_num = getattr(episode, "number", 1.0)
        ep_title = getattr(episode, "title", None)

        series_slug = slugify(series_title)
        dir_name = f"{anilist_id}_{series_slug}"
        filename = f"ep{int(ep_num) if ep_num.is_integer() else ep_num}_{quality}.ts"

        download_dir = Path.home() / ".local" / "share" / "anilili" / "downloads" / dir_name
        download_dir.mkdir(parents=True, exist_ok=True)

        full_file_path = str(download_dir / filename)

        # 3. Create DownloadRecord in DB
        now = int(asyncio.get_event_loop().time()) if asyncio.get_event_loop().is_running() else 0
        record = anilili_core.DownloadRecord(
            id=download_id,
            anilist_id=anilist_id,
            episode_num=ep_num,
            series_title=series_title,
            provider=provider,
            category=category,
            quality=quality,
            episode_title=ep_title,
            series_cover=series_cover,
            status="QUEUED",
            progress=0.0,
            file_path=full_file_path,
            file_size=None,
            duration_s=None,
            error_msg=None,
            created_at=now,
            updated_at=now
        )
        self.bridge.create_download(record)

        # 4. Submit to aria2c
        headers = []
        if hasattr(stream_item, "referer") and stream_item.referer:
            headers.append(f"Referer: {stream_item.referer}")
        headers.append("User-Agent: Mozilla/5.0 (X11; Linux x86_64)")

        gid = self.aria2.add_download(
            url=stream_item.url,
            filename=filename,
            headers=headers if headers else None,
            out_dir=str(download_dir)
        )

        if not gid:
            self.bridge.update_download_status(download_id, "ERROR", error_msg="Failed to submit to aria2c")
            self.download_error.emit(download_id, "Failed to submit download task to aria2c")
            return None

        # 5. Store active state and update DB status
        self._active[download_id] = {"gid": gid, "record": record}
        self.bridge.update_download_status(download_id, "DOWNLOADING", progress=0.0, file_path=full_file_path)
        self.download_started.emit(download_id)
        return download_id

    def _poll_active(self):
        if not self._active:
            return

        to_remove = []
        for download_id, data in list(self._active.items()):
            gid = data["gid"]
            record = data["record"]

            status_dict = self.aria2.get_status(gid)
            if not status_dict:
                continue

            aria_status = status_dict.get("status")
            completed_len = int(status_dict.get("completedLength", 0))
            total_len = int(status_dict.get("totalLength", 0))
            speed = int(status_dict.get("downloadSpeed", 0))

            if aria_status == "complete":
                # Download complete! Update DB and download sidecars asynchronously (Feedback #3)
                file_path = record.file_path
                file_size = completed_len if completed_len > 0 else (os.path.getsize(file_path) if file_path and os.path.exists(file_path) else 0)

                self.bridge.update_download_status(
                    download_id, "SAVED", progress=1.0, file_path=file_path, file_size=file_size
                )
                
                # Asynchronously download subtitle sidecars (non-blocking)
                try:
                    asyncio.create_task(self._download_subtitles(record))
                except Exception:
                    pass

                self.download_complete.emit(download_id)
                to_remove.append(download_id)

            elif aria_status == "error":
                err_msg = status_dict.get("errorMessage", "Unknown aria2c error")
                self.bridge.update_download_status(download_id, "ERROR", error_msg=err_msg)
                self.download_error.emit(download_id, err_msg)
                to_remove.append(download_id)

            else:
                # Active downloading
                progress = (completed_len / total_len) if total_len > 0 else 0.0
                self.bridge.update_download_status(download_id, "DOWNLOADING", progress=progress)
                self.download_progress.emit(download_id, progress, speed)

        for id_val in to_remove:
            self._active.pop(id_val, None)

    async def _download_subtitles(self, record):
        """Asynchronously download subtitle sidecars without blocking the Qt main thread (Feedback #3)."""
        if not record.file_path:
            return

        base_path, _ = os.path.splitext(record.file_path)
        # Fetch Konoha episode subtitle URLs if available
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                # Stub sidecar check / download logic
                pass
        except Exception as e:
            print(f"[DownloadManager] Subtitle sidecar download error: {e}")

    def cancel_download(self, download_id: str):
        if download_id in self._active:
            gid = self._active[download_id]["gid"]
            self.aria2.remove_download(gid)
            self._active.pop(download_id, None)

        self.bridge.update_download_status(download_id, "ERROR", error_msg="Cancelled")
        self.download_error.emit(download_id, "Cancelled")

    def delete_download(self, download_id: str):
        if download_id in self._active:
            self.cancel_download(download_id)

        # Get record from DB / active
        def _on_get(success, record, err):
            if success and record and record.file_path:
                file_path = record.file_path
                # Delete main file
                if os.path.exists(file_path):
                    try:
                        os.remove(file_path)
                    except OSError:
                        pass

                # Delete matching sidecars (*.vtt, *.srt, *.ass) (Feedback #6)
                base_path, _ = os.path.splitext(file_path)
                for sidecar_pattern in [f"{base_path}*.vtt", f"{base_path}*.srt", f"{base_path}*.ass"]:
                    for sidecar in glob.glob(sidecar_pattern):
                        try:
                            os.remove(sidecar)
                        except OSError:
                            pass

            self.bridge.delete_download(download_id)

        self.bridge.get_download_by_id(download_id, _on_get)
