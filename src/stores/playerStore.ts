import { create } from 'zustand';
import type { StreamItem, EpisodeItem, CustomPlaylistItem } from '../types';

interface PlayerState {
  currentAnimeId: number | null;
  currentMalId: number | null;
  currentAnimeTitle: string;
  currentAnimeCover: string | null;
  currentEpisode: EpisodeItem | null;
  episodesList: EpisodeItem[];
  category: 'sub' | 'dub';
  selectedProvider: string;
  currentStream: StreamItem | null;
  availableStreams: StreamItem[];
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  playbackRate: number;
  isFullscreen: boolean;
  autoPlayNext: boolean;
  autoSkipIntroOutro: boolean;

  // Custom Playlist Queue (cross-anime)
  customQueue: CustomPlaylistItem[];
  customQueueIndex: number;
  activePlaylistName: string | null;

  // Subtitle Settings
  subtitleFontSize: number;
  subtitleColor: string;
  subtitleBackground: string;
  subtitleDelayMs: number;

  // Actions
  setEpisode: (
    animeId: number,
    malId: number | null,
    animeTitle: string,
    animeCover: string | null,
    episode: EpisodeItem,
    episodesList: EpisodeItem[],
    category?: 'sub' | 'dub',
    provider?: string
  ) => void;
  setStream: (stream: StreamItem | null, availableStreams?: StreamItem[]) => void;
  setCategory: (category: 'sub' | 'dub') => void;
  setProvider: (provider: string) => void;
  setIsPlaying: (playing: boolean) => void;
  setTime: (current: number, duration: number) => void;
  setVolume: (vol: number) => void;
  setPlaybackRate: (rate: number) => void;
  setIsFullscreen: (fs: boolean) => void;
  setAutoPlayNext: (auto: boolean) => void;
  setAutoSkipIntroOutro: (skip: boolean) => void;
  setSubtitleFontSize: (size: number) => void;
  setSubtitleColor: (color: string) => void;
  setSubtitleBackground: (bg: string) => void;
  setSubtitleDelayMs: (delay: number) => void;
  setCustomQueue: (items: CustomPlaylistItem[], startIndex?: number, playlistName?: string) => void;
  clearCustomQueue: () => void;
  advanceCustomQueue: () => CustomPlaylistItem | null;
  prevCustomQueue: () => CustomPlaylistItem | null;
  jumpToCustomQueueIndex: (index: number) => CustomPlaylistItem | null;
  playNextEpisode: () => EpisodeItem | null;
  playPrevEpisode: () => EpisodeItem | null;
}

export const usePlayerStore = create<PlayerState>((set, get) => ({
  currentAnimeId: null,
  currentMalId: null,
  currentAnimeTitle: '',
  currentAnimeCover: null,
  currentEpisode: null,
  episodesList: [],
  category: 'sub',
  selectedProvider: 'auto',
  currentStream: null,
  availableStreams: [],
  isPlaying: false,
  currentTime: 0,
  duration: 0,
  volume: 1,
  playbackRate: 1,
  isFullscreen: false,
  autoPlayNext: true,
  autoSkipIntroOutro: true,

  subtitleFontSize: 20,
  subtitleColor: '#ffffff',
  subtitleBackground: 'rgba(0, 0, 0, 0.65)',
  subtitleDelayMs: 0,

  customQueue: [],
  customQueueIndex: 0,
  activePlaylistName: null,

  setCustomQueue: (items, startIndex = 0, playlistName) => {
    set({
      customQueue: items,
      customQueueIndex: Math.max(0, Math.min(items.length > 0 ? items.length - 1 : 0, startIndex)),
      activePlaylistName: playlistName || null,
    });
  },

  clearCustomQueue: () => {
    set({
      customQueue: [],
      customQueueIndex: 0,
      activePlaylistName: null,
    });
  },

  advanceCustomQueue: () => {
    const { customQueue, customQueueIndex } = get();
    if (customQueue.length === 0 || customQueueIndex >= customQueue.length - 1) return null;
    const nextIndex = customQueueIndex + 1;
    const nextItem = customQueue[nextIndex];
    set({ customQueueIndex: nextIndex });
    return nextItem;
  },

  prevCustomQueue: () => {
    const { customQueue, customQueueIndex } = get();
    if (customQueue.length === 0 || customQueueIndex <= 0) return null;
    const prevIndex = customQueueIndex - 1;
    const prevItem = customQueue[prevIndex];
    set({ customQueueIndex: prevIndex });
    return prevItem;
  },

  jumpToCustomQueueIndex: (index: number) => {
    const { customQueue } = get();
    if (index >= 0 && index < customQueue.length) {
      set({ customQueueIndex: index });
      return customQueue[index];
    }
    return null;
  },

  setEpisode: (
    animeId,
    malId,
    animeTitle,
    animeCover,
    episode,
    episodesList,
    category = 'sub',
    provider = 'auto'
  ) => {
    set({
      currentAnimeId: animeId,
      currentMalId: malId,
      currentAnimeTitle: animeTitle,
      currentAnimeCover: animeCover,
      currentEpisode: episode,
      episodesList,
      category,
      selectedProvider: provider,
      currentStream: null,
      availableStreams: [],
      currentTime: 0,
      duration: 0,
    });
  },

  setStream: (stream, availableStreams) => {
    set({
      currentStream: stream,
      availableStreams: availableStreams || (stream ? [stream] : []),
    });
  },

  setCategory: (category) => set({ category }),
  setProvider: (selectedProvider) => set({ selectedProvider }),
  setIsPlaying: (isPlaying) => set({ isPlaying }),
  setTime: (currentTime, duration) => set({ currentTime, duration }),
  setVolume: (volume) => set({ volume }),
  setPlaybackRate: (playbackRate) => set({ playbackRate }),
  setIsFullscreen: (isFullscreen) => set({ isFullscreen }),
  setAutoPlayNext: (autoPlayNext) => set({ autoPlayNext }),
  setAutoSkipIntroOutro: (autoSkipIntroOutro) => set({ autoSkipIntroOutro }),
  setSubtitleFontSize: (subtitleFontSize) => set({ subtitleFontSize }),
  setSubtitleColor: (subtitleColor) => set({ subtitleColor }),
  setSubtitleBackground: (subtitleBackground) => set({ subtitleBackground }),
  setSubtitleDelayMs: (subtitleDelayMs) => set({ subtitleDelayMs }),

  playNextEpisode: () => {
    const { currentEpisode, episodesList } = get();
    if (!currentEpisode || episodesList.length === 0) return null;
    const currentIndex = episodesList.findIndex((ep) => ep.number === currentEpisode.number);
    if (currentIndex >= 0 && currentIndex < episodesList.length - 1) {
      const nextEp = episodesList[currentIndex + 1];
      set({ currentEpisode: nextEp, currentStream: null, currentTime: 0 });
      return nextEp;
    }
    return null;
  },

  playPrevEpisode: () => {
    const { currentEpisode, episodesList } = get();
    if (!currentEpisode || episodesList.length === 0) return null;
    const currentIndex = episodesList.findIndex((ep) => ep.number === currentEpisode.number);
    if (currentIndex > 0) {
      const prevEp = episodesList[currentIndex - 1];
      set({ currentEpisode: prevEp, currentStream: null, currentTime: 0 });
      return prevEp;
    }
    return null;
  },
}));
