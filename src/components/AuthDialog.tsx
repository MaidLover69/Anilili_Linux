import React, { useState } from 'react';
import { Dialog } from './Dialog';
import { useUIStore } from '../stores/uiStore';
import { useSettings, useSaveSettings, useAniListViewer } from '../hooks';
import { invoke } from '@tauri-apps/api/core';
import { Check, ExternalLink, LogOut, Key, UserCheck, AlertCircle } from 'lucide-react';

export const AuthDialog: React.FC = () => {
  const { authDialogOpen, setAuthDialogOpen, showToast } = useUIStore();
  const { data: settings } = useSettings();
  const saveSettingsMutation = useSaveSettings();

  const [anilistTokenInput, setAnilistTokenInput] = useState('');
  const [malCodeInput, setMalCodeInput] = useState('');
  const [malVerifier, setMalVerifier] = useState('');
  const [isExchangingMal, setIsExchangingMal] = useState(false);

  const anilistToken = settings?.anilist_token || null;
  const { data: viewer, isLoading: viewerLoading } = useAniListViewer(anilistToken);

  const handleOpenAniListAuth = async () => {
    try {
      const info = await invoke<{ anilist_auth_url: string }>('get_auth_urls');
      window.open(info.anilist_auth_url, '_blank');
      showToast({
        type: 'info',
        title: 'AniList Login Opened',
        message: 'Authorize the application, then copy and paste the access_token from the URL bar below.',
      });
    } catch (e: any) {
      showToast({ type: 'error', title: 'Failed to get auth URL', message: e.toString() });
    }
  };

  const handleSaveAniListToken = async () => {
    if (!anilistTokenInput.trim()) return;
    let token = anilistTokenInput.trim();
    // Support pasting the full redirect URL (http://localhost/#access_token=...&...)
    if (token.includes('access_token=')) {
      const match = token.match(/access_token=([^&]+)/);
      if (match) token = match[1];
    }

    try {
      const viewerData = await invoke<any>('get_anilist_viewer', { token });
      await saveSettingsMutation.mutateAsync({
        anilist_token: token,
        anilist_viewer_id: viewerData.id,
      });
      setAnilistTokenInput('');
      showToast({
        type: 'success',
        title: 'AniList Connected',
        message: `Welcome back, ${viewerData.name}!`,
      });
    } catch (e: any) {
      showToast({
        type: 'error',
        title: 'Authentication Failed',
        message: 'Invalid AniList access token.',
      });
    }
  };

  const handleLogoutAniList = async () => {
    await saveSettingsMutation.mutateAsync({
      anilist_token: null,
      anilist_viewer_id: null,
    });
    showToast({ type: 'info', title: 'AniList Disconnected' });
  };

  return (
    <Dialog
      isOpen={authDialogOpen}
      onClose={() => setAuthDialogOpen(false)}
      title="Account Integrations"
      maxWidth="max-w-md"
    >
      <div className="flex flex-col gap-6">
        {/* AniList Integration */}
        <div className="flex flex-col gap-3 p-4 rounded-xl bg-bg-elevated border border-border">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="w-8 h-8 rounded-lg bg-[#02A9FF]/20 text-[#02A9FF] flex items-center justify-center font-black text-sm">
                AL
              </div>
              <div className="flex flex-col">
                <span className="text-sm font-bold text-text-primary">AniList</span>
                <span className="text-[11px] text-text-muted">
                  Sync watch history, library & scores
                </span>
              </div>
            </div>

            {anilistToken && (
              <span className="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold bg-state-success/20 text-state-success border border-state-success/30">
                <Check className="w-3 h-3 stroke-[3]" /> Connected
              </span>
            )}
          </div>

          {anilistToken && viewer ? (
            <div className="flex items-center justify-between p-3 rounded-lg bg-bg-card border border-border mt-1">
              <div className="flex items-center gap-3">
                {viewer.avatar_url && (
                  <img
                    src={viewer.avatar_url}
                    alt={viewer.name}
                    className="w-10 h-10 rounded-full object-cover border border-border"
                  />
                )}
                <div className="flex flex-col">
                  <span className="text-xs font-bold text-text-primary">{viewer.name}</span>
                  <span className="text-[10px] text-text-muted">
                    {viewer.episodes_watched} episodes • {viewer.anime_count} anime
                  </span>
                </div>
              </div>

              <button
                onClick={handleLogoutAniList}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-bg-elevated text-state-error hover:bg-state-error/20 text-xs font-medium transition-colors"
              >
                <LogOut className="w-3.5 h-3.5" />
                <span>Disconnect</span>
              </button>
            </div>
          ) : (
            <div className="flex flex-col gap-2.5 mt-1">
              <button
                onClick={handleOpenAniListAuth}
                className="flex items-center justify-center gap-2 w-full py-2.5 rounded-xl bg-[#02A9FF] hover:bg-[#0091dc] text-white text-xs font-bold shadow-md transition-all"
              >
                <ExternalLink className="w-3.5 h-3.5" />
                <span>Authorize on AniList.co</span>
              </button>

              <div className="flex items-center gap-2 mt-1">
                <input
                  type="password"
                  placeholder="Paste AniList Access Token or Redirect URL"
                  value={anilistTokenInput}
                  onChange={(e) => setAnilistTokenInput(e.target.value)}
                  className="flex-1 px-3 py-2 rounded-lg bg-bg-input border border-border text-xs text-text-primary placeholder:text-text-muted focus:border-border-focus outline-none"
                />
                <button
                  onClick={handleSaveAniListToken}
                  disabled={!anilistTokenInput.trim()}
                  className="px-3.5 py-2 rounded-lg bg-accent-500 hover:bg-accent-700 disabled:opacity-40 text-white text-xs font-bold transition-colors"
                >
                  Save
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </Dialog>
  );
};
