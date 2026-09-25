import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { CustomPlaylist, CustomPlaylistItem, ExternalPlaylistItem } from '../types';

export function usePlaylists() {
  return useQuery<CustomPlaylist[]>({
    queryKey: ['customPlaylists'],
    queryFn: async () => {
      return await invoke<CustomPlaylist[]>('list_playlists');
    },
  });
}

export function usePlaylistItems(playlistId: string | null) {
  return useQuery<CustomPlaylistItem[]>({
    queryKey: ['customPlaylistItems', playlistId],
    queryFn: async () => {
      if (!playlistId) return [];
      return await invoke<CustomPlaylistItem[]>('get_playlist_items', { playlistId });
    },
    enabled: !!playlistId,
  });
}

export function useCreatePlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ name, description }: { name: string; description?: string }) => {
      return await invoke<CustomPlaylist>('create_playlist', { name, description });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customPlaylists'] });
    },
  });
}

export function useAddToPlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (item: CustomPlaylistItem) => {
      return await invoke('add_to_playlist', { item });
    },
    onSuccess: (_, item) => {
      queryClient.invalidateQueries({ queryKey: ['customPlaylists'] });
      queryClient.invalidateQueries({ queryKey: ['customPlaylistItems', item.playlist_id] });
    },
  });
}

export function useAddMultipleToPlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ playlistId, items }: { playlistId: string; items: CustomPlaylistItem[] }) => {
      return await invoke('add_multiple_to_playlist', { items });
    },
    onSuccess: (_, { playlistId }) => {
      queryClient.invalidateQueries({ queryKey: ['customPlaylists'] });
      queryClient.invalidateQueries({ queryKey: ['customPlaylistItems', playlistId] });
    },
  });
}

export function useRemoveFromPlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ itemId, playlistId }: { itemId: string; playlistId: string }) => {
      return await invoke('remove_from_playlist', { itemId });
    },
    onSuccess: (_, { playlistId }) => {
      queryClient.invalidateQueries({ queryKey: ['customPlaylists'] });
      queryClient.invalidateQueries({ queryKey: ['customPlaylistItems', playlistId] });
    },
  });
}

export function useDeletePlaylist() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (playlistId: string) => {
      return await invoke('delete_playlist', { playlistId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['customPlaylists'] });
    },
  });
}

export async function launchPlaylistExternalPlayer(
  player: 'mpv' | 'vlc',
  playlistName: string,
  items: ExternalPlaylistItem[]
): Promise<boolean> {
  return await invoke<boolean>('launch_playlist_external_player', {
    player,
    playlistName,
    items,
  });
}
