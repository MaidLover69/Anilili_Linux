import sys
import os
os.environ["LC_NUMERIC"] = "C"
os.environ["LC_ALL"] = "C"
import time
import locale
import ctypes
from urllib.parse import urlparse
try:
    locale.setlocale(locale.LC_NUMERIC, "C")
    locale.setlocale(locale.LC_ALL, "C")
    ctypes.CDLL(None).setlocale(1, b"C")
except Exception:
    pass
import mpv
from PyQt6.QtCore import pyqtSignal, pyqtSlot, Qt, QTimer, QMetaObject, Q_ARG, QPoint
from PyQt6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton, QFrame,
    QStackedLayout, QMenu
)
from PyQt6.QtGui import QAction, QCursor
from utils.backend_bridge import BackendBridge
from components.seek_bar import SeekBar
from components.settings_sheet import PlayerSettingsSheet

class MpvWidget(QWidget):
    time_pos_changed = pyqtSignal(float)
    duration_changed = pyqtSignal(float)
    pause_changed = pyqtSignal(bool)
    idle_changed = pyqtSignal(bool)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WidgetAttribute.WA_DontCreateNativeAncestors)
        self.setAttribute(Qt.WidgetAttribute.WA_NativeWindow)
        self.player = None

    def showEvent(self, event):
        super().showEvent(event)
        if self.player is None:
            wid_int = int(self.winId())
            if wid_int == 0:
                # Window not yet mapped; defer init to next event loop tick
                QTimer.singleShot(0, self._init_mpv)
            else:
                self._init_mpv()

    def _init_mpv(self):
        if self.player is not None:
            return
        win_id = str(int(self.winId()))
        try:
            self.player = mpv.MPV(
                wid=win_id,
                vo="gpu",
                hwdec="vaapi",  # Force Mesa VAAPI / suppress libcuda.so.1 fallback
                sub_auto="fuzzy",
                keep_open=True,
                idle=True,
                input_default_bindings=False,
                input_vo_keyboard=False,
                osc=False,
                ytdl=False,
                cache="yes",
                demuxer_max_bytes="50MiB",
            )
        except Exception as ex:
            print(f"[MPV] Primary init failed ({ex}), falling back to software video decoding...")
            self.player = mpv.MPV(
                wid=win_id,
                vo="gpu",
                hwdec="no",  # Software video decode fallback
                sub_auto="fuzzy",
                keep_open=True,
                idle=True,
                input_default_bindings=False,
                input_vo_keyboard=False,
                osc=False,
                ytdl=False,
                cache="yes",
                demuxer_max_bytes="50MiB",
            )

        # Observe properties thread-safely
        @self.player.property_observer('time-pos')
        def _on_time(name, val):
            if val is not None:
                QMetaObject.invokeMethod(self, "_emit_time", Qt.ConnectionType.QueuedConnection, Q_ARG(float, float(val)))

        @self.player.property_observer('duration')
        def _on_dur(name, val):
            if val is not None:
                QMetaObject.invokeMethod(self, "_emit_dur", Qt.ConnectionType.QueuedConnection, Q_ARG(float, float(val)))

        @self.player.property_observer('pause')
        def _on_pause(name, val):
            if val is not None:
                QMetaObject.invokeMethod(self, "_emit_pause", Qt.ConnectionType.QueuedConnection, Q_ARG(bool, bool(val)))

        @self.player.property_observer('core-idle')
        def _on_idle(name, val):
            if val is not None:
                QMetaObject.invokeMethod(self, "_emit_idle", Qt.ConnectionType.QueuedConnection, Q_ARG(bool, bool(val)))

    @pyqtSlot(float)
    def _emit_time(self, val: float):
        self.time_pos_changed.emit(val)

    @pyqtSlot(float)
    def _emit_dur(self, val: float):
        self.duration_changed.emit(val)

    @pyqtSlot(bool)
    def _emit_pause(self, val: bool):
        self.pause_changed.emit(val)

    @pyqtSlot(bool)
    def _emit_idle(self, val: bool):
        self.idle_changed.emit(val)


class ControlsOverlay(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(20, 20, 20, 20)

        # Top Bar
        self.top_bar = QWidget(self)
        top_layout = QHBoxLayout(self.top_bar)
        top_layout.setContentsMargins(0, 0, 0, 0)

        self.back_btn = QPushButton("← Back", self.top_bar)
        self.back_btn.setProperty("class", "secondary")
        self.back_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        top_layout.addWidget(self.back_btn)

        self.title_lbl = QLabel("Anime Title · Episode 1", self.top_bar)
        self.title_lbl.setStyleSheet("color: #ffffff; font-size: 14px; font-weight: bold;")
        top_layout.addWidget(self.title_lbl, stretch=1, alignment=Qt.AlignmentFlag.AlignCenter)

        self.settings_btn = QPushButton("⚙", self.top_bar)
        self.settings_btn.setFixedSize(32, 32)
        self.settings_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.settings_btn.setStyleSheet("color: #ffffff; background: rgba(255,255,255,0.1); border-radius: 6px; font-size: 14px;")
        top_layout.addWidget(self.settings_btn)

        layout.addWidget(self.top_bar)
        layout.addStretch()

        # Skip Intro/Outro Floating Pill
        skip_row = QHBoxLayout()
        skip_row.addStretch()
        self.skip_btn = QPushButton("Skip Intro ›", self)
        self.skip_btn.setProperty("class", "primary")
        self.skip_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.skip_btn.setVisible(False)
        skip_row.addWidget(self.skip_btn)
        layout.addLayout(skip_row)

        layout.addSpacing(12)

        # Bottom Bar
        self.bottom_bar = QWidget(self)
        bottom_layout = QVBoxLayout(self.bottom_bar)
        bottom_layout.setContentsMargins(0, 0, 0, 0)
        bottom_layout.setSpacing(6)

        self.seek_bar = SeekBar(self.bottom_bar)
        bottom_layout.addWidget(self.seek_bar)

        transport_row = QHBoxLayout()
        transport_row.setSpacing(12)

        self.prev_btn = QPushButton("⏮", self.bottom_bar)
        self.prev_btn.setFixedSize(32, 32)
        self.prev_btn.setStyleSheet("color: white; background: transparent; border: none; font-size: 14px;")
        transport_row.addWidget(self.prev_btn)

        self.play_pause_btn = QPushButton("⏵", self.bottom_bar)
        self.play_pause_btn.setFixedSize(36, 36)
        self.play_pause_btn.setStyleSheet("color: white; background: #8979F2; border-radius: 18px; font-size: 16px;")
        transport_row.addWidget(self.play_pause_btn)

        self.next_btn = QPushButton("⏭", self.bottom_bar)
        self.next_btn.setFixedSize(32, 32)
        self.next_btn.setStyleSheet("color: white; background: transparent; border: none; font-size: 14px;")
        transport_row.addWidget(self.next_btn)

        self.time_lbl = QLabel("00:00 / 00:00", self.bottom_bar)
        self.time_lbl.setStyleSheet("color: rgba(255,255,255,0.7); font-size: 12px; font-weight: 500;")
        transport_row.addWidget(self.time_lbl)

        transport_row.addStretch()

        self.fs_btn = QPushButton("⛶", self.bottom_bar)
        self.fs_btn.setFixedSize(32, 32)
        self.fs_btn.setStyleSheet("color: white; background: transparent; border: none; font-size: 14px;")
        transport_row.addWidget(self.fs_btn)

        bottom_layout.addLayout(transport_row)
        layout.addWidget(self.bottom_bar)


class WatchScreen(QWidget):
    exit_requested = pyqtSignal()
    episode_watched = pyqtSignal(int, float)  # anilist_id, ep_num

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setMouseTracking(True)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.bridge = BackendBridge.instance()

        self._anilist_id = None
        self._mal_id = None
        self._episode_number = 1.0
        self._category = "sub"
        self._anime_title = ""
        self._episode_title = ""
        self._duration_s = 0.0
        self._current_pos_s = 0.0
        self._last_saved_pos_ms = 0
        self._marked_watched = False
        self._skip_times = {}

        # Drag gesture tracking
        self._drag_start_pos = None
        self._is_dragging = False

        # Controls auto-hide timer (3s)
        self.hide_timer = QTimer(self)
        self.hide_timer.setInterval(3000)
        self.hide_timer.setSingleShot(True)
        self.hide_timer.timeout.connect(self._hide_controls)

        # Autoplay countdown timer (5s)
        self.autoplay_timer = QTimer(self)
        self.autoplay_timer.setInterval(5000)
        self.autoplay_timer.setSingleShot(True)
        self.autoplay_timer.timeout.connect(self._on_autoplay_complete)

        # Stacked Layout (MPV Surface + Controls Overlay)
        self.stacked_layout = QStackedLayout(self)
        self.stacked_layout.setStackingMode(QStackedLayout.StackingMode.StackAll)

        self.mpv_widget = MpvWidget(self)
        self.mpv_widget.setMouseTracking(True)
        self.mpv_widget.time_pos_changed.connect(self._on_time_pos)
        self.mpv_widget.duration_changed.connect(self._on_duration)
        self.mpv_widget.pause_changed.connect(self._on_pause_change)
        self.mpv_widget.idle_changed.connect(self._on_idle_change)
        self.stacked_layout.addWidget(self.mpv_widget)

        self.controls = ControlsOverlay(self)
        self.controls.setMouseTracking(True)
        self.controls.top_bar.setMouseTracking(True)
        self.controls.bottom_bar.setMouseTracking(True)
        self.controls.back_btn.clicked.connect(self._exit_watch)
        self.controls.play_pause_btn.clicked.connect(self._toggle_play_pause)
        self.controls.fs_btn.clicked.connect(self._toggle_fullscreen)
        self.controls.settings_btn.clicked.connect(self._open_settings)
        self.controls.seek_bar.seek_requested.connect(self._seek_to)
        self.controls.skip_btn.clicked.connect(self._on_skip_clicked)
        self.stacked_layout.addWidget(self.controls)

        # Settings Sheet
        self.settings_sheet = PlayerSettingsSheet(self)
        self.settings_sheet.hide()
        self.settings_sheet.speed_changed.connect(self._set_speed)
        self.settings_sheet.sub_delay_changed.connect(self._set_sub_delay)

        # Backend Bridge listeners
        self.bridge.episode_sources_loaded.connect(self._on_sources_loaded)
        self.bridge.skip_times_loaded.connect(self._on_skip_times_loaded)
        self.bridge.episode_progress_loaded.connect(self._on_progress_loaded)

    def _player(self):
        return getattr(self.mpv_widget, "player", None)

    def _stream_headers(self, stream) -> list[str]:
        headers = ["User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0"]
        referer = getattr(stream, "referer", None)
        if referer:
            headers.append(f"Referer: {referer}")
            origin = getattr(stream, "origin", None)
            if not origin:
                parsed = urlparse(referer)
                if parsed.scheme and parsed.netloc:
                    origin = f"{parsed.scheme}://{parsed.netloc}"
            if origin:
                headers.append(f"Origin: {origin}")
        return headers

    def load_episode(self, anilist_id: int, mal_id: int | None, episode_number: float, category: str = "sub", anime_title: str = "", episode_title: str = "", offline_path: str | None = None, subtitles: list | None = None, title_romaji: str | None = None):
        # 1. Stop previous playback immediately
        player = self._player()
        if player:
            try:
                player.stop()
            except Exception:
                pass

        # 2. Reset playback positions and duration state
        self._anilist_id = anilist_id
        self._mal_id = mal_id
        self._episode_number = episode_number
        self._category = category
        self._anime_title = anime_title
        self._episode_title = episode_title
        self._title_romaji = title_romaji or anime_title
        self._marked_watched = False
        self._current_pos_s = 0.0
        self._duration_s = 0.0
        self._last_saved_pos_ms = 0
        self._skip_times = {}

        # 3. Reset seek bar and UI time label immediately
        self.controls.seek_bar.set_position(0.0)
        self.controls.seek_bar.set_duration(0.0)
        self.controls.seek_bar.set_skip_times({})
        self.controls.time_lbl.setText("00:00 / 00:00")
        self.controls.skip_btn.setVisible(False)

        # 4. Set episode title label
        title_str = f"{anime_title} · Episode {int(episode_number)}" if episode_number.is_integer() else f"{anime_title} · Episode {episode_number}"
        if episode_title:
            title_str += f" · {episode_title}"
        self.controls.title_lbl.setText(title_str)

        # 5. Show controls and request keyboard focus
        self._show_controls()
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setFocus()

        if offline_path and os.path.exists(offline_path):
            # Offline local playback mode
            player = self._player()
            if player:
                try:
                    player.play(offline_path)
                    if subtitles:
                        for sub in subtitles:
                            player.sub_add(sub)
                except Exception as ex:
                    print(f"[Watch] offline playback failed: {ex}")
            self.bridge.load_episode_progress(anilist_id, episode_number)
        else:
            # Online stream resolution mode
            self.bridge.load_episode_sources(anilist_id, mal_id, episode_number, category, title_romaji=self._title_romaji)
            self.bridge.load_episode_progress(anilist_id, episode_number)


    def _on_sources_loaded(self, streams: list):
        print(f"[Watch] sources received: {len(streams)} streams")
        if not streams:
            print("[Watch] NO SOURCES — player will stall")
            return
        stream = streams[0]
        print(f"[Watch] First URL: {stream.url[:80]}")
        player = self._player()
        if player:
            try:
                headers = self._stream_headers(stream)
                referer = getattr(stream, "referer", None)
                if referer:
                    player.referrer = referer
                player["http-header-fields"] = headers
            except Exception as ex:
                print(f"[Watch] Header injection error: {ex}")

            print(f"[Watch] Calling MPV play with URL: {stream.url[:80]}")
            try:
                player.play(stream.url)
            except Exception as ex:
                print(f"[Watch] MPV play failed: {ex}")

        self.settings_sheet.populate_qualities(streams)

    def _on_progress_loaded(self, pos_ms: int, dur_ms: int):
        if pos_ms > 5000 and dur_ms > 0 and pos_ms < (dur_ms - 30000):
            # Resume seek if partially watched (not completed)
            resume_s = pos_ms / 1000.0
            player = self._player()
            if player:
                try:
                    print(f"[Watch] Resuming playback at {resume_s:.1f}s")
                    player.seek(resume_s, reference="absolute")
                except Exception:
                    pass
        else:
            # New or completed episode: start from 0.0s
            player = self._player()
            if player:
                try:
                    player.seek(0.0, reference="absolute")
                except Exception:
                    pass

    def _on_skip_times_loaded(self, skip_dict: dict):
        self._skip_times = skip_dict
        self.controls.seek_bar.set_skip_times(skip_dict)

    def _on_time_pos(self, pos_s: float):
        self._current_pos_s = pos_s
        self.controls.seek_bar.set_position(pos_s)
        self._update_time_label()

        # Save watch progress every 5s
        pos_ms = int(pos_s * 1000)
        if pos_ms - self._last_saved_pos_ms >= 5000:
            self._last_saved_pos_ms = pos_ms
            dur_ms = int(self._duration_s * 1000)
            self.bridge.save_watch_progress(
                anilist_id=self._anilist_id,
                title=self._anime_title,
                cover=None,
                episode_number=self._episode_number,
                episode_title=self._episode_title,
                provider="AniBD",
                category=self._category,
                position_ms=pos_ms,
                duration_ms=dur_ms
            )

        # 85% completion check for auto-mark-watched
        if self._duration_s > 0 and not self._marked_watched:
            if (pos_s / self._duration_s) >= 0.85:
                self._marked_watched = True
                ep_num_int = int(self._episode_number)
                self.episode_watched.emit(self._anilist_id, self._episode_number)
                self.on_episode_watched(self._anilist_id, ep_num_int, self._mal_id)

        # Check AniSkip Intro/Outro pill visibility
        if self._skip_times:
            i_start = self._skip_times.get("intro_start")
            i_end = self._skip_times.get("intro_end")
            o_start = self._skip_times.get("outro_start")
            o_end = self._skip_times.get("outro_end")

            if i_start is not None and i_end is not None and i_start <= pos_s <= i_end:
                self.controls.skip_btn.setText("Skip Intro ›")
                self.controls.skip_btn.skip_target = i_end
                self.controls.skip_btn.setVisible(True)
            elif o_start is not None and o_end is not None and o_start <= pos_s <= o_end:
                self.controls.skip_btn.setText("Skip Outro ›")
                self.controls.skip_btn.skip_target = o_end
                self.controls.skip_btn.setVisible(True)
            else:
                self.controls.skip_btn.setVisible(False)

    def on_episode_watched(self, anilist_id: int, episode_number: int, mal_id: int | None):
        # Fire-and-forget sync to AniList & MAL
        from auth import AniListAuthManager, MalAuthManager
        from settings.store import SettingsStore

        if AniListAuthManager.instance().is_authenticated() and SettingsStore.instance().get("auto_sync_anilist"):
            token = AniListAuthManager.instance().get_token()
            if token and anilist_id:
                self.bridge.save_anilist_progress(token, anilist_id, episode_number)

        if MalAuthManager.instance().is_authenticated() and mal_id:
            mal_token = MalAuthManager.instance().get_access_token()
            if mal_token:
                self.bridge.save_mal_progress(mal_token, mal_id, episode_number)


    def _on_duration(self, dur_s: float):
        self._duration_s = dur_s
        self.controls.seek_bar.set_duration(dur_s)
        self._update_time_label()

        # Load AniSkip times once duration is known (> 60s)
        if dur_s > 60.0 and self._mal_id:
            self.bridge.load_skip_times(self._mal_id, self._episode_number, dur_s)

    def _on_pause_change(self, paused: bool):
        self.controls.play_pause_btn.setText("⏵" if paused else "⏸")
        if paused:
            self._show_controls()

    def _on_idle_change(self, idle: bool):
        pass

    def _update_time_label(self):
        curr_min, curr_sec = int(self._current_pos_s // 60), int(self._current_pos_s % 60)
        dur_min, dur_sec = int(self._duration_s // 60), int(self._duration_s % 60)
        self.controls.time_lbl.setText(f"{curr_min:02d}:{curr_sec:02d} / {dur_min:02d}:{dur_sec:02d}")

    def _toggle_play_pause(self):
        player = self._player()
        if player:
            try:
                paused = getattr(player, 'pause', False)
                print(f"[Watch] toggle play, currently paused={paused}")
                player.pause = not paused
            except Exception as ex:
                print(f"[Watch] _toggle_play_pause failed: {ex}")

    def _seek_to(self, target_s: float):
        player = self._player()
        if player:
            try:
                player.seek(target_s, reference="absolute")
            except Exception as ex:
                print(f"[Watch] _seek_to failed: {ex}")

    def _on_skip_clicked(self):
        player = self._player()
        if hasattr(self.controls.skip_btn, 'skip_target') and player:
            try:
                player.seek(self.controls.skip_btn.skip_target, reference="absolute")
            except Exception:
                pass

    def _set_speed(self, speed_val: float):
        player = self._player()
        if player:
            try:
                player.speed = speed_val
            except Exception:
                pass

    def _set_sub_delay(self, delay_val: float):
        player = self._player()
        if player:
            try:
                curr = getattr(player, 'sub_delay', 0.0) or 0.0
                player.sub_delay = curr + delay_val
            except Exception:
                pass

    def _open_settings(self):
        self.settings_sheet.show_sheet(self.rect())

    def _show_controls(self):
        if not self.controls.isVisible():
            self.controls.show()
        self.hide_timer.start(3000)

    def _hide_controls(self):
        try:
            player = getattr(self.mpv_widget, 'player', None)
            if player:
                is_paused = getattr(player, 'pause', True)
                if not is_paused and not self.settings_sheet.isVisible():
                    self.controls.hide()
        except Exception:
            pass

    def mouseMoveEvent(self, event):
        self._show_controls()
        self.hide_timer.start(3000)
        super().mouseMoveEvent(event)

    def mouseDoubleClickEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self._toggle_fullscreen()
        super().mouseDoubleClickEvent(event)

    def showEvent(self, event):
        super().showEvent(event)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setFocus()
        if hasattr(self, 'controls'):
            self.controls.raise_()

    def keyPressEvent(self, event):
        print(f"[Watch] keyPressEvent: key={event.key()} text={event.text()}")
        self._show_controls()
        key = event.key()
        handled = True
        if key in (Qt.Key.Key_Space, Qt.Key.Key_K):
            self._toggle_play_pause()
        elif key == Qt.Key.Key_Left:
            if event.modifiers() & Qt.KeyboardModifier.ShiftModifier:
                self._seek_to(max(0.0, self._current_pos_s - 30.0))
            else:
                self._seek_to(max(0.0, self._current_pos_s - 5.0))
        elif key == Qt.Key.Key_Right:
            if event.modifiers() & Qt.KeyboardModifier.ShiftModifier:
                self._seek_to(min(self._duration_s, self._current_pos_s + 30.0))
            else:
                self._seek_to(min(self._duration_s, self._current_pos_s + 5.0))
        elif key == Qt.Key.Key_Up:
            player = self._player()
            if player:
                try:
                    vol = getattr(player, 'volume', 100) or 100
                    player.volume = min(150, vol + 5)
                except Exception:
                    pass
        elif key == Qt.Key.Key_Down:
            player = self._player()
            if player:
                try:
                    vol = getattr(player, 'volume', 100) or 100
                    player.volume = max(0, vol - 5)
                except Exception:
                    pass
        elif key == Qt.Key.Key_M:
            player = self._player()
            if player:
                try:
                    mute = getattr(player, 'mute', False)
                    player.mute = not mute
                except Exception:
                    pass
        elif key == Qt.Key.Key_F:
            self._toggle_fullscreen()
        elif key == Qt.Key.Key_Escape:
            if self.window().isFullScreen():
                self.window().showNormal()
            else:
                self._exit_watch()
        else:
            handled = False

        if handled:
            event.accept()
            return
        super().keyPressEvent(event)

    def _toggle_fullscreen(self):
        win = self.window()
        if win.isFullScreen():
            win.showNormal()
        else:
            win.showFullScreen()

    def _exit_watch(self):
        player = self._player()
        if player:
            try:
                player.pause = True
            except Exception:
                pass
        self.exit_requested.emit()

    def _on_autoplay_complete(self):
        pass
