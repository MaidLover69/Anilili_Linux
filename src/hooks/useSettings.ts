import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, UpdateInfo } from '../types';

export const DEFAULT_SETTINGS: AppSettings = {
  autoplay: true,
  auto_skip_intro_outro: false,
  prefer_dub: false,
  subtitles_with_dub: false,
  default_quality: 'highest',
  player_gestures: true,
  caption_text_scale: 100,
  caption_text_color: 'white',
  caption_background_color: 'black',
  caption_background_opacity: 60,
  caption_bold_text: true,
  caption_bottom_margin: 12,
  caption_edge_style: 'none',
  persistent_caption_delays: '',
  download_quality: 'best',
  download_destination: 'app_only',
  download_dir: '',
  hide_adult_content: true,
  blur_episode_thumbnails: false,
  auto_sync_anilist: true,
  sync_watchlist_to_anilist: true,
  release_notifications: true,
  episode_layout: 'list',
  sidebar_expanded: true,
  menu_language: 'system',
  preferred_provider: 'auto',
  server_priority: '',
  last_pipe_origin: '',
  enable_adult_providers: false,
  update_check_on_launch: true,
  external_player: 'builtin',
};

export function useSettings() {
  return useQuery<AppSettings>({
    queryKey: ['settings'],
    queryFn: async () => {
      const stored = await invoke<Partial<AppSettings>>('get_settings');
      return { ...DEFAULT_SETTINGS, ...stored };
    },
    staleTime: Infinity,
  });
}

export function useSaveSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (settings: Partial<AppSettings>) => {
      const current = queryClient.getQueryData<AppSettings>(['settings']) || DEFAULT_SETTINGS;
      const merged = { ...current, ...settings };
      await invoke('save_settings', { settings: merged });
      return merged;
    },
    onSuccess: (merged) => {
      queryClient.setQueryData(['settings'], merged);
    },
  });
}

export function useCheckUpdate() {
  return useQuery<UpdateInfo | null>({
    queryKey: ['checkUpdate'],
    queryFn: async () => {
      return await invoke<UpdateInfo | null>('check_for_update');
    },
    staleTime: 60 * 60 * 1000,
  });
}
