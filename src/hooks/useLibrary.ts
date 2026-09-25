import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { HistoryEntry, WatchlistEntry, MediaListEntry } from '../types';

export function useWatchlist() {
  return useQuery<WatchlistEntry[]>({
    queryKey: ['watchlist'],
    queryFn: async () => {
      return await invoke<WatchlistEntry[]>('get_watchlist');
    },
  });
}

export function useIsInWatchlist(anilistId: number | null) {
  return useQuery<boolean>({
    queryKey: ['isInWatchlist', anilistId],
    queryFn: async () => {
      if (!anilistId) return false;
      return await invoke<boolean>('is_in_watchlist', { anilistId });
    },
    enabled: !!anilistId,
  });
}

export function useWatchedEpisodes(anilistId: number | null) {
  return useQuery<number[]>({
    queryKey: ['watchedEpisodes', anilistId],
    queryFn: async () => {
      if (!anilistId) return [];
      return await invoke<number[]>('get_watched_episodes', { anilistId });
    },
    enabled: !!anilistId,
  });
}

export function useEpisodeProgress(anilistId: number | null, episodeNumber: number | null) {
  return useQuery<[number, number] | null>({
    queryKey: ['episodeProgress', anilistId, episodeNumber],
    queryFn: async () => {
      if (!anilistId || episodeNumber === null) return null;
      return await invoke<[number, number] | null>('get_episode_progress', {
        anilistId,
        episodeNumber,
      });
    },
    enabled: !!anilistId && episodeNumber !== null,
  });
}

export function useContinueWatching() {
  return useQuery<HistoryEntry[]>({
    queryKey: ['continueWatching'],
    queryFn: async () => {
      return await invoke<HistoryEntry[]>('get_continue_watching');
    },
  });
}

export function useToggleWatchlist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      anilistId,
      title,
      cover,
      format,
      averageScore,
      inWatchlist,
    }: {
      anilistId: number;
      title: string;
      cover?: string | null;
      format?: string | null;
      averageScore?: number | null;
      inWatchlist: boolean;
    }) => {
      if (inWatchlist) {
        await invoke('remove_from_watchlist', { anilistId });
      } else {
        await invoke('add_to_watchlist', {
          anilistId,
          title,
          cover: cover || null,
          format: format || null,
          averageScore: averageScore !== undefined ? averageScore : null,
        });
      }
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['watchlist'] });
      queryClient.invalidateQueries({ queryKey: ['isInWatchlist', variables.anilistId] });
    },
  });
}

export function useUpdateWatchProgress() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      anilistId,
      title,
      cover,
      episodeNumber,
      episodeTitle,
      provider,
      category,
      positionMs,
      durationMs,
    }: {
      anilistId: number;
      title: string;
      cover?: string | null;
      episodeNumber: number;
      episodeTitle?: string | null;
      provider: string;
      category: string;
      positionMs: number;
      durationMs: number;
    }) => {
      await invoke('update_watch_progress', {
        anilistId,
        title,
        cover: cover || null,
        episodeNumber,
        episodeTitle: episodeTitle || null,
        provider,
        category,
        positionMs,
        durationMs,
      });
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['continueWatching'] });
      queryClient.invalidateQueries({ queryKey: ['watchedEpisodes', variables.anilistId] });
      queryClient.invalidateQueries({
        queryKey: ['episodeProgress', variables.anilistId, variables.episodeNumber],
      });
    },
  });
}

export function useRemoteLibrary(status?: string) {
  return useQuery<MediaListEntry[]>({
    queryKey: ['remoteLibrary', status],
    queryFn: async () => {
      return await invoke<MediaListEntry[]>('get_library_entries', {
        status: status || null,
      });
    },
  });
}
