import os
import shutil
import socket
import subprocess
import time
import httpx
from pathlib import Path

ARIA2_RPC_URL = "http://localhost:6800/jsonrpc"
RPC_SECRET = "anilili_secret"

class Aria2Manager:
    _instance = None

    @classmethod
    def instance(cls) -> "Aria2Manager":
        if cls._instance is None:
            cls._instance = cls()
        return cls._instance

    def __init__(self):
        self._process = None

    def is_port_open(self, port: int = 6800) -> bool:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(0.5)
            return s.connect_ex(("localhost", port)) == 0

    def start_aria2c(self) -> bool:
        # Check port collision / existing session first (Feedback #2)
        if self.is_port_open(6800):
            print("[Aria2Manager] Port 6800 is already open. Using existing aria2c instance.")
            return True

        aria2c_path = shutil.which("aria2c")
        if not aria2c_path:
            print("[Aria2Manager] aria2c executable not found in PATH.")
            return False

        download_dir = Path.home() / ".local" / "share" / "anilili" / "downloads"
        download_dir.mkdir(parents=True, exist_ok=True)

        cmd = [
            aria2c_path,
            "--enable-rpc",
            "--rpc-listen-port=6800",
            f"--rpc-secret={RPC_SECRET}",
            f"--dir={download_dir}",
            "--max-concurrent-downloads=3",
            "--split=8",
            "--max-connection-per-server=8",
            "--continue=true",
            "--quiet=true",
        ]

        try:
            self._process = subprocess.Popen(cmd)
            # Wait briefly for startup
            for _ in range(10):
                if self.is_port_open(6800):
                    print("[Aria2Manager] aria2c RPC server started successfully.")
                    return True
                time.sleep(0.1)
        except Exception as e:
            print(f"[Aria2Manager] Failed to start aria2c subprocess: {e}")

        return False

    def stop_aria2c(self):
        if self._process:
            try:
                self._process.terminate()
                self._process.wait(timeout=2.0)
            except Exception:
                pass
            self._process = None

    def _rpc_call(self, method: str, params: list) -> dict | None:
        payload = {
            "jsonrpc": "2.0",
            "id": "anilili",
            "method": method,
            "params": [f"token:{RPC_SECRET}"] + params,
        }

        # Retry up to 3 times for RPC startup (per spec)
        for attempt in range(3):
            try:
                with httpx.Client(timeout=2.0) as client:
                    res = client.post(ARIA2_RPC_URL, json=payload)
                    if res.status_code == 200:
                        return res.json().get("result")
            except Exception:
                time.sleep(0.5)

        return None

    def add_download(self, url: str, filename: str, headers: list[str] | None = None, out_dir: str | None = None) -> str | None:
        options = {"out": filename}
        if out_dir:
            options["dir"] = out_dir
        if headers:
            options["header"] = headers

        res = self._rpc_call("aria2.addUri", [[url], options])
        if res and isinstance(res, str):
            return res
        return None

    def get_status(self, gid: str) -> dict | None:
        return self._rpc_call("aria2.tellStatus", [gid])

    def remove_download(self, gid: str) -> bool:
        res = self._rpc_call("aria2.remove", [gid])
        return res is not None
