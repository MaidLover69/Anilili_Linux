import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { DownloadRecord, StorageCheck } from '../types';

export function useAllDownloads() {
  return useQuery<DownloadRecord[]>({
    queryKey: ['allDownloads'],
    queryFn: async () => {
      return await invoke<DownloadRecord[]>('get_all_downloads');
    },
    refetchInterval: 3000,
  });
}

export function useDownloadsForAnime(anilistId: number | null) {
  return useQuery<DownloadRecord[]>({
    queryKey: ['downloads', anilistId],
    queryFn: async () => {
      if (!anilistId) return [];
      return await invoke<DownloadRecord[]>('get_downloads', { anilistId });
    },
    enabled: !!anilistId,
    refetchInterval: 3000,
  });
}

export function useCheckStorage(quality: string = '720p') {
  return useQuery<StorageCheck>({
    queryKey: ['checkStorage', quality],
    queryFn: async () => {
      return await invoke<StorageCheck>('check_storage_for_download', { quality });
    },
  });
}

export function useCreateDownload() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (record: DownloadRecord) => {
      return await invoke<string>('create_download', { record });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['allDownloads'] });
      queryClient.invalidateQueries({ queryKey: ['downloads'] });
    },
  });
}

export function useDeleteDownload() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      await invoke('delete_download', { id });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['allDownloads'] });
      queryClient.invalidateQueries({ queryKey: ['downloads'] });
    },
  });
}
