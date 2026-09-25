import os
import re
import shutil
import glob
import asyncio
from pathlib import Path
from utils.backend_bridge import BackendBridge

class EpisodeExporter:
    @staticmethod
    def check_ffmpeg() -> bool:
        return shutil.which("ffmpeg") is not None

    @staticmethod
    def _parse_time_to_seconds(line_str: str) -> float | None:
        # Match time=HH:MM:SS.ss or time=MM:SS.ss
        match = re.search(r"time=(\d+):(\d+):(\d+\.\d+)", line_str)
        if match:
            h, m, s = match.groups()
            return float(h) * 3600 + float(m) * 60 + float(s)

        match2 = re.search(r"time=(\d+):(\d+\.\d+)", line_str)
        if match2:
            m, s = match2.groups()
            return float(m) * 60 + float(s)
        return None

    async def export_to_mp4(self, download_id: str, output_dir: str, progress_callback=None) -> str | None:
        if not self.check_ffmpeg():
            print("[EpisodeExporter] ffmpeg not found in PATH")
            return None

        bridge = BackendBridge.instance()
        loop = asyncio.get_event_loop()
        record_future = loop.create_future()

        def _on_rec(success, rec, err):
            if not record_future.done():
                record_future.set_result(rec if success else None)

        bridge.get_download_by_id(download_id, _on_rec)
        record = await record_future

        if not record or not record.file_path or not os.path.exists(record.file_path):
            print("[EpisodeExporter] Download record or file path invalid")
            return None

        bridge.update_download_status(download_id, "CONVERTING")

        try:
            ep_num = int(record.episode_num) if record.episode_num.is_integer() else record.episode_num
            out_folder = Path(output_dir) / record.series_title
            out_folder.mkdir(parents=True, exist_ok=True)

            out_filename = f"Episode {ep_num} [{record.quality}].mp4"
            out_path = str(out_folder / out_filename)

            cmd = [
                "ffmpeg",
                "-i", record.file_path,
                "-c", "copy",
                "-movflags", "+faststart",
                out_path,
                "-y"
            ]

            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )

            # Parse stderr for time=HH:MM:SS.ss progress (Feedback #5)
            duration_s = record.duration_s
            while True:
                line = await process.stderr.readline()
                if not line:
                    break
                line_str = line.decode("utf-8", errors="ignore")
                if "time=" in line_str and progress_callback:
                    parsed_sec = self._parse_time_to_seconds(line_str)
                    if parsed_sec is not None and duration_s and duration_s > 0:
                        frac = min(1.0, parsed_sec / duration_s)
                        progress_callback(frac)
                    else:
                        progress_callback(0.5)

            await process.wait()

            # Copy subtitle sidecars
            base_path, _ = os.path.splitext(record.file_path)
            for sidecar_pattern in [f"{base_path}*.vtt", f"{base_path}*.srt", f"{base_path}*.ass"]:
                for sidecar in glob.glob(sidecar_pattern):
                    try:
                        shutil.copy(sidecar, out_folder)
                    except Exception:
                        pass

            bridge.update_download_status(download_id, "SAVED")
            return out_path

        except Exception as ex:
            print(f"[EpisodeExporter] Export failed: {ex}")
            bridge.update_download_status(download_id, "SAVED")
            return None
