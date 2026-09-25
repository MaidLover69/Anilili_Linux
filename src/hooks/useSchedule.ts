import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { AiringEntry, NotificationPreference } from '../types';

export function useAiringSchedule(fromTs: number, toTs: number) {
  return useQuery<AiringEntry[]>({
    queryKey: ['airingSchedule', fromTs, toTs],
    queryFn: async () => {
      return await invoke<AiringEntry[]>('fetch_airing_schedule', { fromTs, toTs });
    },
    staleTime: 30 * 60 * 1000,
  });
}

export function useNotificationPreferences() {
  return useQuery<NotificationPreference[]>({
    queryKey: ['notificationPreferences'],
    queryFn: async () => {
      return await invoke<NotificationPreference[]>('list_notification_preferences');
    },
  });
}

export function useToggleNotificationPreference() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (pref: NotificationPreference) => {
      await invoke('save_notification_preference', { pref });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notificationPreferences'] });
    },
  });
}
