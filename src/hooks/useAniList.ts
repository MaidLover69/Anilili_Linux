import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Media, HomeData, Viewer, MediaListEntry } from '../types';

export function useHomeData() {
  return useQuery<HomeData>({
    queryKey: ['homeData'],
    queryFn: async () => {
      return await invoke<HomeData>('fetch_home_data');
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

export function useTrending() {
  return useQuery<Media[]>({
    queryKey: ['trending'],
    queryFn: async () => {
      return await invoke<Media[]>('fetch_trending');
    },
    staleTime: 10 * 60 * 1000,
  });
}

export function usePopular() {
  return useQuery<Media[]>({
    queryKey: ['popular'],
    queryFn: async () => {
      return await invoke<Media[]>('fetch_popular');
    },
    staleTime: 10 * 60 * 1000,
  });
}

export function useTopRated() {
  return useQuery<Media[]>({
    queryKey: ['topRated'],
    queryFn: async () => {
      return await invoke<Media[]>('fetch_top_rated');
    },
    staleTime: 10 * 60 * 1000,
  });
}

export interface SearchParams {
  query?: string;
  genres?: string[];
  format?: string;
  status?: string;
  sort?: string;
  page?: number;
  perPage?: number;
}

export function useSearchAnime(params: SearchParams, options?: { enabled?: boolean }) {
  return useQuery<Media[]>({
    queryKey: ['searchAnime', params],
    queryFn: async () => {
      return await invoke<Media[]>('search_anime', {
        query: params.query || null,
        genres: params.genres && params.genres.length > 0 ? params.genres : null,
        format: params.format || null,
        status: params.status || null,
        sort: params.sort || 'POPULARITY_DESC',
        page: params.page || 1,
        perPage: params.perPage || 24,
      });
    },
    enabled: options?.enabled,
    staleTime: 2 * 60 * 1000,
  });
}

export function useAnimeDetails(id: number | null) {
  return useQuery<any>({
    queryKey: ['animeDetails', id],
    queryFn: async () => {
      if (!id) return null;
      return await invoke<any>('fetch_anime_details', { id });
    },
    enabled: !!id,
    staleTime: 15 * 60 * 1000,
  });
}

export function useMalAnimeDetails(malId: number | null | undefined) {
  return useQuery<any>({
    queryKey: ['malAnimeDetails', malId],
    queryFn: async () => {
      if (!malId) return null;
      return await invoke<any>('fetch_mal_anime_synopsis', { malId });
    },
    enabled: !!malId,
    staleTime: 60 * 60 * 1000,
  });
}

export function useAnimeThemes(malId: number | null | undefined) {
  return useQuery<import('../types').AnimeThemeEntry[]>({
    queryKey: ['animeThemes', malId],
    queryFn: async () => {
      if (!malId) return [];
      return await invoke<import('../types').AnimeThemeEntry[]>('fetch_anime_themes', { malId });
    },
    enabled: !!malId,
    staleTime: 24 * 60 * 60 * 1000,
  });
}

export function useAniListViewer(token: string | null) {
  return useQuery<Viewer | null>({
    queryKey: ['aniListViewer', token],
    queryFn: async () => {
      if (!token) return null;
      return await invoke<Viewer>('get_anilist_viewer', { token });
    },
    enabled: !!token,
    staleTime: 30 * 60 * 1000,
  });
}

export function useAniListLibrary(token: string | null, userId: number | null, status?: string) {
  return useQuery<MediaListEntry[]>({
    queryKey: ['aniListLibrary', token, userId, status],
    queryFn: async () => {
      if (!token || !userId) return [];
      return await invoke<MediaListEntry[]>('fetch_anilist_library', {
        token,
        userId,
        status: status || null,
      });
    },
    enabled: !!token && !!userId,
  });
}

export function useSaveAniListEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      token,
      mediaId,
      status,
      progress,
      score,
    }: {
      token: string;
      mediaId: number;
      status?: string;
      progress?: number;
      score?: number;
    }) => {
      return await invoke<MediaListEntry>('save_anilist_entry', {
        token,
        mediaId,
        status: status || null,
        progress: progress !== undefined ? progress : null,
        score: score !== undefined ? score : null,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['aniListLibrary'] });
    },
  });
}
