import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { EpisodesResult, SourcesResult, SkipTimes, EpisodeItem } from '../types';

export function useAnimeCatalogEpisodes(
  anilistId: number | null,
  malId?: number | null,
  totalEpisodes?: number | null,
  titleRomaji?: string | null,
  status?: string | null,
  nextAiringEpisode?: number | null
) {
  return useQuery<EpisodeItem[]>({
    queryKey: ['animeCatalogEpisodes', anilistId, malId, totalEpisodes, titleRomaji, status, nextAiringEpisode],
    queryFn: async () => {
      if (!anilistId) return [];
      return await invoke<EpisodeItem[]>('get_anime_catalog_episodes', {
        anilistId,
        malId: malId || null,
        totalEpisodes: totalEpisodes || null,
        titleRomaji: titleRomaji || null,
        status: status || null,
        nextAiringEpisode: nextAiringEpisode || null,
      });
    },
    enabled: !!anilistId,
    staleTime: 30 * 60 * 1000,
  });
}

export function useEpisodes(
  anilistId: number | null,
  malId?: number | null,
  titleRomaji?: string | null
) {
  return useQuery<EpisodesResult>({
    queryKey: ['episodes', anilistId, malId, titleRomaji],
    queryFn: async () => {
      if (!anilistId) return {};
      return await invoke<EpisodesResult>('get_all_episodes', {
        anilistId,
        malId: malId || null,
        titleRomaji: titleRomaji || null,
      });
    },
    enabled: !!anilistId,
    staleTime: 10 * 60 * 1000,
  });
}

export function useEpisodeSources(
  anilistId: number | null,
  malId: number | null | undefined,
  episodeNumber: number | null,
  category: 'sub' | 'dub',
  titleRomaji?: string | null,
  enabled: boolean = true
) {
  return useQuery<SourcesResult>({
    queryKey: ['episodeSources', anilistId, malId, episodeNumber, category, titleRomaji],
    queryFn: async () => {
      if (!anilistId || episodeNumber === null) {
        return { streams: [], subtitles: [], skip: null };
      }
      return await invoke<SourcesResult>('get_episode_sources', {
        anilistId,
        malId: malId || null,
        episodeNumber,
        category,
        titleRomaji: titleRomaji || null,
      });
    },
    enabled: enabled && !!anilistId && episodeNumber !== null,
    staleTime: 5 * 60 * 1000,
    retry: 2,
  });
}

export function useSkipTimes(
  malId: number | null | undefined,
  episodeNumber: number | null,
  durationS: number = 1440
) {
  return useQuery<SkipTimes>({
    queryKey: ['skipTimes', malId, episodeNumber, durationS],
    queryFn: async () => {
      if (!malId || episodeNumber === null) {
        return { intro_start: null, intro_end: null, outro_start: null, outro_end: null };
      }
      return await invoke<SkipTimes>('get_skip_times', {
        malId,
        episodeNumber,
        durationS,
      });
    },
    enabled: !!malId && episodeNumber !== null,
    staleTime: 60 * 60 * 1000,
  });
}

export function useKonohaEpisodes(anilistId: number | null) {
  return useQuery<EpisodeItem[]>({
    queryKey: ['konohaEpisodes', anilistId],
    queryFn: async () => {
      if (!anilistId) return [];
      return await invoke<EpisodeItem[]>('get_konoha_episodes', { anilistId });
    },
    enabled: !!anilistId,
    staleTime: 30 * 60 * 1000,
  });
}

export function useFillerEpisodes(malId: number | null | undefined) {
  return useQuery<number[]>({
    queryKey: ['fillerEpisodes', malId],
    queryFn: async () => {
      if (!malId) return [];
      return await invoke<number[]>('get_filler_episodes', { malId });
    },
    enabled: !!malId,
    staleTime: 24 * 60 * 60 * 1000,
  });
}
