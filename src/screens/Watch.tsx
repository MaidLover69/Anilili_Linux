import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useParams, useNavigate, useSearchParams } from 'react-router-dom';
import Hls from 'hls.js';
import {
  Play,
  Pause,
  Volume2,
  VolumeX,
  Maximize,
  Minimize,
  SkipForward,
  SkipBack,
  RotateCcw,
  RotateCw,
  Settings,
  ArrowLeft,
  Tv,
  Radio,
  ExternalLink,
  FastForward,
  Check,
  Download,
  Sliders,
  MessageSquare,
  ListOrdered,
  Layers,
} from 'lucide-react';
import { ServerSelectDialog } from '../components/ServerSelectDialog';
import { SubtitleSettingsModal } from '../components/SubtitleSettingsModal';
import { PlayerPlaylistDrawer } from '../components/PlayerPlaylistDrawer';
import { CustomSelect } from '../components/CustomSelect';
import {
  useAnimeDetails,
  useAnimeCatalogEpisodes,
  useEpisodes,
  useEpisodeSources,
  useSkipTimes,
  useUpdateWatchProgress,
  useSettings,
} from '../hooks';
import { usePlayerStore } from '../stores/playerStore';
import { useUIStore } from '../stores/uiStore';
import { invoke } from '@tauri-apps/api/core';
import type { EpisodeItem as EpisodeItemType, StreamItem, SubtitleItem } from '../types';

export const Watch: React.FC = () => {
  const { id, episodeNum } = useParams<{ id: string; episodeNum: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();

  const anilistId = id ? parseInt(id, 10) : 0;
  const currentEpNumber = episodeNum ? parseFloat(episodeNum) : 1;

  const urlCategory = (searchParams.get('cat') as 'sub' | 'dub') || 'sub';
  const urlProvider = searchParams.get('prov') || 'auto';

  const [category, setCategory] = useState<'sub' | 'dub'>(urlCategory);
  const [provider, setProvider] = useState<string>(urlProvider);

  // Settings
  const { data: appSettings } = useSettings();
  const { showToast } = useUIStore();

  // Anime Details & Episodes
  const { data: details } = useAnimeDetails(anilistId);
  const malId = details?.idMal || null;
  const titleRomaji = details?.title?.romaji || null;
  const animeTitle =
    details?.title?.english ||
    details?.title?.userPreferred ||
    details?.title?.romaji ||
    'Anime';
  const animeCover =
    details?.coverImage?.extraLarge || details?.coverImage?.large || null;

  const { data: episodesData = {} } = useEpisodes(anilistId, malId, titleRomaji);
  const providerNames = Object.keys(episodesData);
  const activeProviderName = provider !== 'auto' && episodesData[provider]
    ? provider
    : providerNames[0] || 'Senshi';

  const { data: catalogEpisodes = [] } = useAnimeCatalogEpisodes(
    anilistId,
    malId,
    details?.episodes,
    titleRomaji,
    details?.status,
    details?.nextAiringEpisode?.episode
  );

  const currentProviderData = episodesData[activeProviderName] || { sub: [], dub: [] };
  const rawEpisodesList =
    category === 'dub' ? currentProviderData.dub : currentProviderData.sub;

  const currentEpisodesList =
    rawEpisodesList && rawEpisodesList.length > 0
      ? rawEpisodesList
      : catalogEpisodes;

  // Sources query
  const { data: sources, isLoading: sourcesLoading } = useEpisodeSources(
    anilistId,
    malId,
    currentEpNumber,
    category,
    titleRomaji
  );

  const [selectedQuality, setSelectedQuality] = useState<string>('auto');
  const [activeStream, setActiveStream] = useState<StreamItem | null>(null);

  // Video Element & HLS ref
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const hlsRef = useRef<Hls | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

  // Player State
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(1);
  const [isMuted, setIsMuted] = useState(false);
  const [playbackRate, setPlaybackRate] = useState(1);
  const {
    autoSkipIntroOutro,
    autoPlayNext,
    subtitleFontSize,
    subtitleColor,
    subtitleBackground,
    subtitleDelayMs,
    customQueue,
    customQueueIndex,
    activePlaylistName,
    advanceCustomQueue,
    prevCustomQueue,
    clearCustomQueue,
  } = usePlayerStore();

  const [isFullscreen, setIsFullscreen] = useState(false);
  const [controlsVisible, setControlsVisible] = useState(true);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [isSubtitleSettingsOpen, setIsSubtitleSettingsOpen] = useState(false);
  const [isPlaylistDrawerOpen, setIsPlaylistDrawerOpen] = useState(false);
  const [qualityLevels, setQualityLevels] = useState<{ id: number; height: number; name: string }[]>([]);
  const [currentQualityLevel, setCurrentQualityLevel] = useState<number>(-1); // -1 = auto

  // Subtitle & Audio Tracks state
  const [selectedSubtitleUrl, setSelectedSubtitleUrl] = useState<string>('off');
  const [audioTracks, setAudioTracks] = useState<{ id: number; name: string; lang: string }[]>([]);
  const [selectedAudioTrackId, setSelectedAudioTrackId] = useState<number>(-1);

  // Playback Queue (multi-selected episodes)
  const [playbackQueue, setPlaybackQueue] = useState<number[]>([]);

  const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const progressSaveRef = useRef<number>(0);
  const [isServerDialogOpen, setIsServerDialogOpen] = useState(false);

  // Sync Discord Rich Presence
  useEffect(() => {
    if (animeTitle) {
      invoke('update_discord_presence', {
        animeTitle,
        episodeNumber: currentEpNumber,
        isPlaying,
      }).catch(() => {});
    }
    return () => {
      invoke('clear_discord_presence').catch(() => {});
    };
  }, [animeTitle, currentEpNumber, isPlaying]);

  // Skip Times
  const { data: skipTimes } = useSkipTimes(malId, currentEpNumber, duration || 1440);
  const [showSkipButton, setShowSkipButton] = useState<'intro' | 'outro' | null>(null);
  const [seekFeedback, setSeekFeedback] = useState<{ side: 'left' | 'right'; text: string } | null>(null);
  const seekFeedbackTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const triggerSeek = (seconds: number) => {
    if (!videoRef.current) return;
    videoRef.current.currentTime = Math.max(0, Math.min(duration || 99999, videoRef.current.currentTime + seconds));
    const side = seconds > 0 ? 'right' : 'left';
    const text = seconds > 0 ? `+${seconds}s` : `${seconds}s`;
    setSeekFeedback({ side, text });
    if (seekFeedbackTimeoutRef.current) clearTimeout(seekFeedbackTimeoutRef.current);
    seekFeedbackTimeoutRef.current = setTimeout(() => setSeekFeedback(null), 650);
  };

  // Navigation handlers
  const handleSelectEpisode = (epNum: number, targetAnilistId?: number, targetCategory?: 'sub' | 'dub') => {
    const aid = targetAnilistId || anilistId;
    const cat = targetCategory || category;
    navigate(`/watch/${aid}/${epNum}?cat=${cat}&prov=${provider}`);
  };

  const handlePrevEpisode = () => {
    if (customQueue.length > 0 && customQueueIndex > 0) {
      const prevItem = prevCustomQueue();
      if (prevItem) {
        handleSelectEpisode(prevItem.episode_num, prevItem.anilist_id, (prevItem.category as 'sub' | 'dub') || category);
        showToast({ type: 'info', title: `Playing: ${prevItem.series_title} - Ep ${prevItem.episode_num}` });
        return;
      }
    }
    if (currentEpNumber > 1) {
      handleSelectEpisode(currentEpNumber - 1);
    }
  };

  const handleNextEpisode = () => {
    if (customQueue.length > 0 && customQueueIndex < customQueue.length - 1) {
      const nextItem = advanceCustomQueue();
      if (nextItem) {
        handleSelectEpisode(nextItem.episode_num, nextItem.anilist_id, (nextItem.category as 'sub' | 'dub') || category);
        showToast({ type: 'info', title: `Playing: ${nextItem.series_title} - Ep ${nextItem.episode_num}` });
        return;
      }
    }
    if (playbackQueue.length > 0) {
      const [nextEp, ...rest] = playbackQueue;
      setPlaybackQueue(rest);
      handleSelectEpisode(nextEp);
    } else {
      handleSelectEpisode(currentEpNumber + 1);
    }
  };

  // Queue manipulation
  const handleToggleQueueEpisode = (epNum: number) => {
    if (playbackQueue.includes(epNum)) {
      setPlaybackQueue(playbackQueue.filter((n) => n !== epNum));
    } else {
      setPlaybackQueue([...playbackQueue, epNum]);
    }
  };

  const handlePlayQueue = () => {
    if (playbackQueue.length > 0) {
      const [firstEp, ...rest] = playbackQueue;
      setPlaybackQueue(rest);
      handleSelectEpisode(firstEp);
      showToast({ type: 'success', title: `Playing queue: starting Episode ${firstEp}` });
    }
  };

  const handleClearQueue = () => {
    setPlaybackQueue([]);
    showToast({ type: 'info', title: 'Playback queue cleared' });
  };

  // DB progress update mutation
  const updateProgressMutation = useUpdateWatchProgress();

  // Download Current Episode with MKV & Subtitles
  const handleDownloadCurrentEpisode = () => {
    if (!activeStream) {
      showToast({ type: 'error', title: 'Stream is resolving. Please wait...' });
      return;
    }

    const subsList = sources?.subtitles || [];
    invoke('start_episode_download', {
      record: {
        id: `dl-${anilistId}-${currentEpNumber}-${Date.now()}`,
        anilist_id: anilistId,
        episode_num: currentEpNumber,
        episode_title: `Episode ${currentEpNumber}`,
        series_title: animeTitle,
        series_cover: animeCover,
        provider: activeProviderName,
        category,
        quality: activeStream.quality || 'auto',
        status: 'QUEUED',
        progress: 0.0,
        file_path: null,
        file_size: null,
        duration_s: duration || null,
        error_msg: null,
        created_at: Math.floor(Date.now() / 1000),
        updated_at: Math.floor(Date.now() / 1000),
      },
      streamUrl: activeStream.url,
      referer: activeStream.referer || null,
      origin: activeStream.origin || null,
      subtitles: subsList,
    })
      .then(() => {
        showToast({
          type: 'success',
          title: `Downloading Episode ${currentEpNumber} (MKV + Subtitles)`,
        });
      })
      .catch((err) => {
        showToast({ type: 'error', title: `Download error: ${err}` });
      });
  };

  // Select stream when sources change
  useEffect(() => {
    if (sources && sources.streams.length > 0) {
      const stream = sources.streams[0];
      setActiveStream(stream);

      // Auto select default subtitle if available
      if (sources.subtitles && sources.subtitles.length > 0) {
        const defSub = sources.subtitles.find((s) => s.is_default) || sources.subtitles[0];
        setSelectedSubtitleUrl(defSub.url);
      } else {
        setSelectedSubtitleUrl('off');
      }
    } else {
      setActiveStream(null);
      setSelectedSubtitleUrl('off');
    }
  }, [sources]);

  // HLS stream setup with referer/origin injection & multi-audio tracking
  useEffect(() => {
    const video = videoRef.current;
    if (!video || !activeStream) return;

    if (hlsRef.current) {
      hlsRef.current.destroy();
      hlsRef.current = null;
    }

    const streamUrl = activeStream.url;
    const isHls =
      activeStream.stream_type === 'hls' ||
      streamUrl.includes('.m3u8') ||
      streamUrl.includes('/hls/');

    if (isHls && Hls.isSupported()) {
      const hls = new Hls({
        xhrSetup: (xhr, url) => {
          if (activeStream.referer) {
            xhr.setRequestHeader('Referer', activeStream.referer);
          }
          if (activeStream.origin) {
            xhr.setRequestHeader('Origin', activeStream.origin);
          }
          if (activeStream.headers) {
            for (const [k, v] of Object.entries(activeStream.headers)) {
              xhr.setRequestHeader(k, v);
            }
          }
        },
        enableWorker: true,
        lowLatencyMode: false,
        maxBufferLength: 30,
        maxBufferSize: 60 * 1024 * 1024,
        backBufferLength: 90,
      });

      hls.loadSource(streamUrl);
      hls.attachMedia(video);

      hls.on(Hls.Events.MANIFEST_PARSED, (_, data) => {
        const levels = data.levels.map((lvl, index) => ({
          id: index,
          height: lvl.height,
          name: lvl.height ? `${lvl.height}p` : `Level ${index + 1}`,
        }));
        setQualityLevels(levels);
        if (appSettings?.autoplay) {
          video.play().catch(() => {});
        }
      });

      hls.on(Hls.Events.AUDIO_TRACKS_UPDATED, (_, data) => {
        const tracks = data.audioTracks.map((trk) => ({
          id: trk.id,
          name: trk.name || trk.lang || `Track ${trk.id + 1}`,
          lang: trk.lang || '',
        }));
        setAudioTracks(tracks);
      });

      hls.on(Hls.Events.LEVEL_SWITCHED, (_, data) => {
        setCurrentQualityLevel(data.level);
      });

      hls.on(Hls.Events.ERROR, (_, errData) => {
        if (errData.fatal) {
          switch (errData.type) {
            case Hls.ErrorTypes.NETWORK_ERROR:
              hls.startLoad();
              break;
            case Hls.ErrorTypes.MEDIA_ERROR:
              hls.recoverMediaError();
              break;
            default:
              hls.destroy();
              break;
          }
        }
      });

      hlsRef.current = hls;
    } else {
      video.src = streamUrl;
      if (appSettings?.autoplay) {
        video.play().catch(() => {});
      }
    }

    return () => {
      if (hlsRef.current) {
        hlsRef.current.destroy();
        hlsRef.current = null;
      }
    };
  }, [activeStream, appSettings?.autoplay]);

  // Audio track switching handler
  const handleSelectAudioTrack = (trackId: number) => {
    setSelectedAudioTrackId(trackId);
    if (hlsRef.current) {
      hlsRef.current.audioTrack = trackId;
      showToast({ type: 'info', title: `Switched audio track` });
    }
  };

  // Subtitle track selection handler
  const handleSelectSubtitle = (url: string) => {
    setSelectedSubtitleUrl(url);
    const video = videoRef.current;
    if (!video) return;

    // Toggle track visibility
    const textTracks = video.textTracks;
    for (let i = 0; i < textTracks.length; i++) {
      const track = textTracks[i];
      if (url === 'off') {
        track.mode = 'disabled';
      } else {
        // Match by track src
        track.mode = 'showing';
      }
    }

    if (url === 'off') {
      showToast({ type: 'info', title: 'Subtitles turned off' });
    } else {
      const subObj = sources?.subtitles.find((s) => s.url === url);
      showToast({
        type: 'success',
        title: `Subtitle: ${subObj?.language || 'Enabled'}`,
      });
    }
  };

  // Time update, AniSkip auto-skipping, and progress tracking
  const handleTimeUpdate = () => {
    const video = videoRef.current;
    if (!video) return;

    const time = video.currentTime;
    setCurrentTime(time);

    // Check AniSkip
    if (skipTimes) {
      if (skipTimes.intro_start !== null && skipTimes.intro_end !== null) {
        if (time >= skipTimes.intro_start && time < skipTimes.intro_end) {
          if (autoSkipIntroOutro) {
            video.currentTime = skipTimes.intro_end;
            showToast({ type: 'info', title: 'Auto-skipped Intro' });
          } else {
            setShowSkipButton('intro');
          }
        } else if (showSkipButton === 'intro') {
          setShowSkipButton(null);
        }
      }

      if (skipTimes.outro_start !== null && skipTimes.outro_end !== null) {
        if (time >= skipTimes.outro_start && time < skipTimes.outro_end) {
          if (autoSkipIntroOutro) {
            video.currentTime = skipTimes.outro_end;
            showToast({ type: 'info', title: 'Auto-skipped Outro' });
          } else {
            setShowSkipButton('outro');
          }
        } else if (showSkipButton === 'outro') {
          setShowSkipButton(null);
        }
      }
    }

    // Save progress every 5 seconds
    const now = Date.now();
    if (now - progressSaveRef.current > 5000) {
      progressSaveRef.current = now;
      updateProgressMutation.mutate({
        anilistId,
        episodeNumber: currentEpNumber,
        episodeTitle: `Episode ${currentEpNumber}`,
        provider: activeProviderName,
        category,
        positionMs: Math.floor(time * 1000),
        durationMs: Math.floor((duration || video.duration || 1) * 1000),
        title: animeTitle,
        cover: animeCover,
      });
    }
  };

  const handleLoadedMetadata = () => {
    const video = videoRef.current;
    if (!video) return;
    setDuration(video.duration);
  };

  const handleEnded = () => {
    setIsPlaying(false);
    if (playbackQueue.length > 0) {
      const [nextEp, ...rest] = playbackQueue;
      setPlaybackQueue(rest);
      handleSelectEpisode(nextEp);
      showToast({ type: 'info', title: `Queue: Playing Episode ${nextEp}` });
    } else if (autoPlayNext) {
      handleNextEpisode();
      showToast({ type: 'info', title: `Auto-playing Episode ${currentEpNumber + 1}` });
    }
  };

  const togglePlay = () => {
    const video = videoRef.current;
    if (!video) return;
    if (isPlaying) {
      video.pause();
    } else {
      video.play().catch(() => {});
    }
    setIsPlaying(!isPlaying);
  };

  const toggleMute = () => {
    const video = videoRef.current;
    if (!video) return;
    video.muted = !isMuted;
    setIsMuted(!isMuted);
  };

  const handleVolumeChange = (newVol: number) => {
    const video = videoRef.current;
    if (!video) return;
    video.volume = newVol;
    setVolume(newVol);
    setIsMuted(newVol === 0);
  };

  const handleSeek = (newTime: number) => {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = newTime;
    setCurrentTime(newTime);
  };

  const toggleFullscreen = () => {
    const container = containerRef.current;
    if (!container) return;
    if (!document.fullscreenElement) {
      container.requestFullscreen().catch(() => {});
      setIsFullscreen(true);
    } else {
      document.exitFullscreen().catch(() => {});
      setIsFullscreen(false);
    }
  };

  const handleSkipAction = () => {
    const video = videoRef.current;
    if (!video || !skipTimes) return;
    if (showSkipButton === 'intro' && skipTimes.intro_end !== null) {
      video.currentTime = skipTimes.intro_end;
      setShowSkipButton(null);
    } else if (showSkipButton === 'outro' && skipTimes.outro_end !== null) {
      video.currentTime = skipTimes.outro_end;
      setShowSkipButton(null);
    }
  };

  // Keyboard Shortcuts (including Shift+N / Shift+P)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't capture shortcuts if typing in input
      if (
        document.activeElement?.tagName === 'INPUT' ||
        document.activeElement?.tagName === 'TEXTAREA'
      ) {
        return;
      }

      switch (e.key) {
        case ' ':
        case 'k':
        case 'K':
          e.preventDefault();
          togglePlay();
          break;
        case 'f':
        case 'F':
          e.preventDefault();
          toggleFullscreen();
          break;
        case 'm':
        case 'M':
          e.preventDefault();
          toggleMute();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          triggerSeek(e.shiftKey ? -30 : -5);
          break;
        case 'ArrowRight':
          e.preventDefault();
          triggerSeek(e.shiftKey ? 30 : 5);
          break;
        case 'ArrowUp':
          e.preventDefault();
          handleVolumeChange(Math.min(1, volume + 0.05));
          break;
        case 'ArrowDown':
          e.preventDefault();
          handleVolumeChange(Math.max(0, volume - 0.05));
          break;
        case 'N':
          if (e.shiftKey) {
            e.preventDefault();
            handleNextEpisode();
          }
          break;
        case 'P':
          if (e.shiftKey) {
            e.preventDefault();
            handlePrevEpisode();
          }
          break;
        case 'Escape':
          if (isFullscreen) {
            toggleFullscreen();
          }
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isPlaying, volume, isFullscreen, currentEpNumber, playbackQueue]);

  // Mouse movement control visibility
  const handleMouseMove = () => {
    setControlsVisible(true);
    if (controlsTimeoutRef.current) {
      clearTimeout(controlsTimeoutRef.current);
    }
    controlsTimeoutRef.current = setTimeout(() => {
      if (isPlaying) {
        setControlsVisible(false);
      }
    }, 3000);
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  };

  return (
    <div
      ref={containerRef}
      onMouseMove={handleMouseMove}
      className="relative w-full h-full min-h-[500px] aspect-video bg-black rounded-2xl overflow-hidden shadow-2xl flex items-center justify-center select-none group"
    >
      {/* Video Element */}
      <video
        ref={videoRef}
        onTimeUpdate={handleTimeUpdate}
        onLoadedMetadata={handleLoadedMetadata}
        onEnded={handleEnded}
        onPlay={() => setIsPlaying(true)}
        onPause={() => setIsPlaying(false)}
        onClick={togglePlay}
        className="w-full h-full object-contain cursor-pointer"
        playsInline
      >
        {/* Render available subtitle tracks */}
        {sources?.subtitles?.map((sub, idx) => (
          <track
            key={idx}
            src={sub.url}
            kind="subtitles"
            srcLang={sub.language}
            label={sub.language}
            default={sub.is_default || selectedSubtitleUrl === sub.url}
          />
        ))}
      </video>

      {/* Dynamic Subtitle Style Injection */}
      <style>{`
        video::cue {
          font-size: ${subtitleFontSize}px !important;
          color: ${subtitleColor} !important;
          background-color: ${subtitleBackground} !important;
          font-family: inherit !important;
          text-shadow: 0 0 4px #000, 0 0 8px #000 !important;
          border-radius: 4px !important;
        }
      `}</style>

      {/* Seek Feedback Indicators */}
      {seekFeedback && (
        <div
          className={`absolute top-1/2 -translate-y-1/2 ${
            seekFeedback.side === 'left' ? 'left-16' : 'right-16'
          } p-4 rounded-2xl bg-black/60 backdrop-blur-md border border-white/10 flex items-center gap-2 text-white font-black text-xl animate-fade-in pointer-events-none`}
        >
          {seekFeedback.side === 'left' ? (
            <RotateCcw className="w-6 h-6 text-accent-400" />
          ) : (
            <RotateCw className="w-6 h-6 text-accent-400" />
          )}
          <span>{seekFeedback.text}</span>
        </div>
      )}

      {/* Floating AniSkip Pill */}
      {showSkipButton && (
        <button
          onClick={handleSkipAction}
          className="absolute right-8 bottom-24 z-30 px-5 py-2.5 rounded-full bg-accent-500 hover:bg-accent-600 text-white font-extrabold text-xs shadow-2xl border border-white/20 flex items-center gap-2 animate-bounce cursor-pointer"
        >
          <FastForward className="w-4 h-4 fill-current" />
          <span>
            {showSkipButton === 'intro' ? 'Skip Opening (OP) ›' : 'Skip Ending (ED) ›'}
          </span>
        </button>
      )}

      {/* Controls Overlay */}
      <div
        className={`absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/60 flex flex-col justify-between p-4 transition-opacity duration-300 pointer-events-none ${
          controlsVisible || !isPlaying ? 'opacity-100' : 'opacity-0'
        }`}
      >
        {/* Top Bar */}
        <div className="flex items-center justify-between pointer-events-auto">
          <div className="flex items-center gap-3">
            <button
              onClick={() => navigate(-1)}
              className="p-2 rounded-xl bg-black/40 hover:bg-black/60 text-white border border-white/10 transition-colors cursor-pointer"
              title="Back"
            >
              <ArrowLeft className="w-5 h-5" />
            </button>
            <div className="flex flex-col">
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-bold text-white truncate max-w-md">
                  {animeTitle}
                </h2>
                {activePlaylistName && customQueue.length > 0 && (
                  <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-accent-500/20 text-accent-300 border border-accent-500/30 flex items-center gap-1">
                    <ListOrdered className="w-3 h-3" />
                    {activePlaylistName} ({customQueueIndex + 1}/{customQueue.length})
                  </span>
                )}
              </div>
              <span className="text-xs text-accent-300 font-semibold">
                Episode {currentEpNumber} • {category.toUpperCase()} •{' '}
                {activeProviderName}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* Playlist Drawer Button */}
            <button
              onClick={() => setIsPlaylistDrawerOpen(true)}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-black/40 hover:bg-black/60 border border-white/10 text-white text-xs font-bold transition-all cursor-pointer"
              title="Episode Playlist & Queue"
            >
              <ListOrdered className="w-4 h-4 text-accent-300" />
              <span>Episodes</span>
              {playbackQueue.length > 0 && (
                <span className="px-1.5 py-0.2 rounded-full bg-accent-500 text-[10px] text-white">
                  {playbackQueue.length}
                </span>
              )}
            </button>

            {/* Subtitle & Audio Settings Button */}
            <button
              onClick={() => setIsSubtitleSettingsOpen(true)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-xl border text-xs font-bold transition-all cursor-pointer ${
                selectedSubtitleUrl !== 'off'
                  ? 'bg-accent-500/20 text-accent-300 border-accent-500/40'
                  : 'bg-black/40 hover:bg-black/60 text-white border-white/10'
              }`}
              title="Subtitles & Audio Settings"
            >
              <MessageSquare className="w-4 h-4" />
              <span>Subtitles</span>
              {selectedSubtitleUrl !== 'off' && (
                <span className="w-1.5 h-1.5 rounded-full bg-accent-400" />
              )}
            </button>

            {/* Server Select Modal Trigger */}
            <button
              onClick={() => setIsServerDialogOpen(true)}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-black/40 hover:bg-black/60 border border-white/10 text-white text-xs font-bold transition-colors cursor-pointer"
            >
              <Radio className="w-4 h-4 text-accent-300" />
              <span>Change Server</span>
            </button>

            {/* Download Episode Button */}
            <button
              onClick={handleDownloadCurrentEpisode}
              className="p-2 rounded-xl bg-black/40 hover:bg-black/60 border border-white/10 text-white hover:text-accent-300 transition-colors cursor-pointer"
              title="Download Episode (MKV with embedded subtitles)"
            >
              <Download className="w-4 h-4" />
            </button>

            {/* External Player */}
            <button
              onClick={() => {
                if (activeStream?.url) {
                  invoke('launch_external_player', {
                    streamUrl: activeStream.url,
                    player: appSettings?.external_player || 'mpv',
                  }).catch((err) => showToast({ type: 'error', title: `External player error: ${err}` }));
                }
              }}
              className="p-2 rounded-xl bg-black/40 hover:bg-black/60 border border-white/10 text-white hover:text-accent-300 transition-colors cursor-pointer"
              title="Open in External Player (MPV/VLC)"
            >
              <ExternalLink className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Bottom Controls Bar */}
        <div className="flex flex-col gap-2 pointer-events-auto">
          {/* Progress / Seek Bar */}
          <div className="flex items-center gap-3">
            <span className="text-xs font-mono font-semibold text-white/80 w-12">
              {formatTime(currentTime)}
            </span>
            <input
              type="range"
              min={0}
              max={duration || 100}
              step={0.1}
              value={currentTime}
              onChange={(e) => handleSeek(parseFloat(e.target.value))}
              className="flex-1 h-1.5 bg-white/20 rounded-lg appearance-none cursor-pointer accent-accent-500 hover:h-2.5 transition-all"
            />
            <span className="text-xs font-mono font-semibold text-white/80 w-12 text-right">
              {formatTime(duration)}
            </span>
          </div>

          {/* Action Row */}
          <div className="flex items-center justify-between pt-1">
            <div className="flex items-center gap-2">
              {/* Previous Episode Button */}
              <button
                onClick={handlePrevEpisode}
                disabled={currentEpNumber <= 1}
                className="p-2 rounded-xl bg-black/30 hover:bg-white/10 disabled:opacity-30 text-white transition-colors cursor-pointer"
                title="Previous Episode (Shift+P)"
              >
                <SkipBack className="w-5 h-5 fill-current" />
              </button>

              {/* Play / Pause */}
              <button
                onClick={togglePlay}
                className="p-3 rounded-full bg-accent-500 hover:bg-accent-600 active:scale-95 text-white shadow-lg transition-all cursor-pointer"
                title="Play/Pause (Space)"
              >
                {isPlaying ? (
                  <Pause className="w-5 h-5 fill-current" />
                ) : (
                  <Play className="w-5 h-5 fill-current ml-0.5" />
                )}
              </button>

              {/* Next Episode Button */}
              <button
                onClick={handleNextEpisode}
                className="p-2 rounded-xl bg-black/30 hover:bg-white/10 text-white transition-colors cursor-pointer"
                title={
                  playbackQueue.length > 0
                    ? `Next in Queue: Ep ${playbackQueue[0]} (Shift+N)`
                    : `Next Episode: Ep ${currentEpNumber + 1} (Shift+N)`
                }
              >
                <SkipForward className="w-5 h-5 fill-current" />
              </button>

              {/* Seek -10s / +10s */}
              <button
                onClick={() => triggerSeek(-10)}
                className="p-2 rounded-xl bg-black/30 hover:bg-white/10 text-white transition-colors cursor-pointer"
                title="Rewind 10s (Left Arrow)"
              >
                <RotateCcw className="w-4 h-4" />
              </button>
              <button
                onClick={() => triggerSeek(10)}
                className="p-2 rounded-xl bg-black/30 hover:bg-white/10 text-white transition-colors cursor-pointer"
                title="Forward 10s (Right Arrow)"
              >
                <RotateCw className="w-4 h-4" />
              </button>

              {/* Volume */}
              <div className="flex items-center gap-1.5 ml-2 group/vol">
                <button
                  onClick={toggleMute}
                  className="p-2 rounded-xl bg-black/30 hover:bg-white/10 text-white transition-colors cursor-pointer"
                >
                  {isMuted || volume === 0 ? (
                    <VolumeX className="w-4 h-4 text-red-400" />
                  ) : (
                    <Volume2 className="w-4 h-4" />
                  )}
                </button>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={isMuted ? 0 : volume}
                  onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
                  className="w-16 h-1 bg-white/30 rounded-lg appearance-none cursor-pointer accent-accent-400 group-hover/vol:w-24 transition-all"
                />
              </div>
            </div>

            <div className="flex items-center gap-2">
              {/* Playback Speed */}
              <CustomSelect
                placement="up"
                value={playbackRate.toString()}
                onChange={(val) => {
                  const r = parseFloat(val);
                  setPlaybackRate(r);
                  if (videoRef.current) videoRef.current.playbackRate = r;
                }}
                options={[
                  { value: '0.5', label: '0.5x' },
                  { value: '0.75', label: '0.75x' },
                  { value: '1', label: '1.0x (Normal)' },
                  { value: '1.25', label: '1.25x' },
                  { value: '1.5', label: '1.5x' },
                  { value: '2', label: '2.0x' },
                  { value: '3', label: '3.0x' },
                  { value: '4', label: '4.0x' },
                ]}
              />

              {/* Quality Selector */}
              {qualityLevels.length > 0 && (
                <CustomSelect
                  placement="up"
                  value={currentQualityLevel.toString()}
                  onChange={(val) => {
                    const lvl = parseInt(val, 10);
                    setCurrentQualityLevel(lvl);
                    if (hlsRef.current) {
                      hlsRef.current.currentLevel = lvl;
                    }
                  }}
                  options={[
                    { value: '-1', label: 'Auto (Best)' },
                    ...qualityLevels.map((lvl) => ({
                      value: lvl.id.toString(),
                      label: lvl.name,
                    })),
                  ]}
                />
              )}

              {/* Fullscreen */}
              <button
                onClick={toggleFullscreen}
                className="p-2 rounded-xl bg-black/30 hover:bg-white/10 text-white transition-colors cursor-pointer"
                title="Fullscreen (F)"
              >
                {isFullscreen ? (
                  <Minimize className="w-4 h-4" />
                ) : (
                  <Maximize className="w-4 h-4" />
                )}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Subtitle & Audio Settings Modal */}
      <SubtitleSettingsModal
        isOpen={isSubtitleSettingsOpen}
        onClose={() => setIsSubtitleSettingsOpen(false)}
        subtitles={sources?.subtitles || []}
        selectedSubtitleUrl={selectedSubtitleUrl}
        onSelectSubtitle={handleSelectSubtitle}
        audioTracks={audioTracks}
        selectedAudioTrackId={selectedAudioTrackId}
        onSelectAudioTrack={handleSelectAudioTrack}
        category={category}
        onToggleCategory={(newCat) => {
          setCategory(newCat);
          setSearchParams({ cat: newCat, prov: provider });
        }}
      />

      {/* Episode Playlist & Multi-Select Queue Drawer */}
      <PlayerPlaylistDrawer
        isOpen={isPlaylistDrawerOpen}
        onClose={() => setIsPlaylistDrawerOpen(false)}
        episodes={currentEpisodesList}
        currentEpisodeNumber={currentEpNumber}
        onSelectEpisode={handleSelectEpisode}
        queuedEpisodes={playbackQueue}
        onToggleQueueEpisode={handleToggleQueueEpisode}
        onPlayQueue={handlePlayQueue}
        onClearQueue={handleClearQueue}
      />

      {/* Server Selection Dialog */}
      <ServerSelectDialog
        isOpen={isServerDialogOpen}
        onClose={() => setIsServerDialogOpen(false)}
        anilistId={anilistId}
        animeTitle={animeTitle}
        episode={{
          number: currentEpNumber,
          pipe_id: `ep-${currentEpNumber}`,
          title: `Episode ${currentEpNumber}`,
          image: animeCover,
          filler: false,
        }}
      />
    </div>
  );
};
