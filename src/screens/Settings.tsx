import React from 'react';
import {
  Settings as SettingsIcon,
  Play,
  Volume2,
  Tv,
  HardDrive,
  Cloud,
  Sparkles,
  Shield,
  Palette,
  ExternalLink,
  Check,
} from 'lucide-react';
import { CustomSelect } from '../components/CustomSelect';
import { useSettings, useSaveSettings, useCheckUpdate } from '../hooks';
import { useUIStore } from '../stores/uiStore';
import type { AppSettings } from '../types';

export const Settings: React.FC = () => {
  const { data: settings } = useSettings();
  const saveSettingsMutation = useSaveSettings();
  const { setAuthDialogOpen, setUpdateDialogOpen, showToast } = useUIStore();
  const { data: updateInfo, refetch: checkUpdateNow } = useCheckUpdate();

  if (!settings) return null;

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    const updated = { ...settings, [key]: value };
    saveSettingsMutation.mutate(updated, {
      onSuccess: () => showToast({ type: 'success', title: 'Settings saved successfully' }),
      onError: (err: any) => showToast({ type: 'error', title: `Failed to save settings: ${err}` }),
    });
  };

  return (
    <div className="flex flex-col gap-6 max-w-4xl pb-16 animate-fade-in select-none">
      {/* Header */}
      <div className="flex items-center gap-3 p-5 rounded-2xl bg-bg-card border border-border shadow">
        <div className="p-2.5 rounded-xl bg-accent-tint text-accent-300 border border-border-focus">
          <SettingsIcon className="w-6 h-6" />
        </div>
        <div>
          <h1 className="text-xl font-bold text-text-primary">Settings & Preferences</h1>
          <p className="text-xs text-text-muted mt-0.5">
            Configure application behavior, playback defaults, and integrations
          </p>
        </div>
      </div>

      {/* Section: Playback & Player */}
      <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow">
        <div className="flex items-center gap-2 text-sm font-bold text-text-primary pb-2 border-b border-border">
          <Play className="w-4 h-4 text-accent-300" />
          <span>Playback & Video Player</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Autoplay */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Autoplay Video</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Automatically start playback when opening an episode
              </span>
            </div>
            <input
              type="checkbox"
              checked={settings.autoplay}
              onChange={(e) => updateSetting('autoplay', e.target.checked)}
              className="w-4 h-4 accent-accent-500 cursor-pointer"
            />
          </div>

          {/* Auto Skip Intro / Outro */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Auto-Skip Openings & Endings</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Automatically jump past intro and outro themes using AniSkip
              </span>
            </div>
            <input
              type="checkbox"
              checked={settings.auto_skip_intro_outro}
              onChange={(e) => updateSetting('auto_skip_intro_outro', e.target.checked)}
              className="w-4 h-4 accent-accent-500 cursor-pointer"
            />
          </div>

          {/* Audio Language Priority */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Audio Language Priority</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Default audio track for newly opened anime
              </span>
            </div>
            <CustomSelect
              value={settings.prefer_dub ? 'dub' : 'sub'}
              onChange={(val) => updateSetting('prefer_dub', val === 'dub')}
              options={[
                { value: 'sub', label: 'Subtitled (Japanese)' },
                { value: 'dub', label: 'Dubbed (English)' },
              ]}
            />
          </div>

          {/* External Player */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Default Player</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Select built-in web player or external desktop player
              </span>
            </div>
            <CustomSelect
              value={settings.external_player || 'builtin'}
              onChange={(val) => updateSetting('external_player', val as any)}
              options={[
                { value: 'builtin', label: 'Built-in Player' },
                { value: 'mpv', label: 'MPV Player' },
                { value: 'vlc', label: 'VLC Media Player' },
              ]}
            />
          </div>
        </div>
      </div>

      {/* Section: Provider Settings */}
      <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow">
        <div className="flex items-center gap-2 text-sm font-bold text-text-primary pb-2 border-b border-border">
          <Tv className="w-4 h-4 text-accent-300" />
          <span>Streaming Providers & Sources</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Preferred Provider */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Preferred Anime Provider</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Primary provider used for fast episode resolution
              </span>
            </div>
            <CustomSelect
              value={settings.preferred_provider || 'auto'}
              onChange={(val) => updateSetting('preferred_provider', val)}
              options={[
                { value: 'auto', label: 'Auto (Best Quality)' },
                { value: 'Senshi', label: 'Senshi' },
                { value: 'KickAssAnime', label: 'KickAssAnime' },
                { value: 'AniBD', label: 'AniBD' },
                { value: 'AnimeKai', label: 'AnimeKai' },
                { value: 'AniDBApp', label: 'AniDBApp' },
                { value: 'AnimeGG', label: 'AnimeGG' },
                { value: 'AnimeShqip', label: 'AnimeShqip' },
                { value: 'RareAnimes', label: 'RareAnimes' },
                { value: 'Anikoto', label: 'Anikoto' },
                { value: 'AniZone', label: 'AniZone' },
              ]}
            />
          </div>

          {/* Hide 18+ Content */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Hide Adult Content (18+)</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Filter out hentai and explicit titles from searches and rails
              </span>
            </div>
            <input
              type="checkbox"
              checked={settings.hide_adult_content}
              onChange={(e) => updateSetting('hide_adult_content', e.target.checked)}
              className="w-4 h-4 accent-accent-500 cursor-pointer"
            />
          </div>
        </div>
      </div>

      {/* Section: Network & Anti-Censorship (DoH) */}
      <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow">
        <div className="flex items-center gap-2 text-sm font-bold text-text-primary pb-2 border-b border-border">
          <Shield className="w-4 h-4 text-accent-300" />
          <span>Network & Anti-Censorship (DNS-over-HTTPS)</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Enable DoH */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">Enable DNS-over-HTTPS (DoH)</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Bypasses ISP domain blocking and DNS poisoning on anime streams
              </span>
            </div>
            <input
              type="checkbox"
              checked={settings.enable_doh ?? true}
              onChange={(e) => updateSetting('enable_doh', e.target.checked)}
              className="w-4 h-4 accent-accent-500 cursor-pointer"
            />
          </div>

          {/* DoH Provider */}
          <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
            <div className="flex flex-col pr-2">
              <span className="text-xs font-bold text-text-primary">DoH Resolver Provider</span>
              <span className="text-[11px] text-text-muted mt-0.5">
                Encrypted DNS resolver endpoint
              </span>
            </div>
            <CustomSelect
              value={settings.doh_provider || 'cloudflare'}
              onChange={(val) => updateSetting('doh_provider', val)}
              options={[
                { value: 'cloudflare', label: 'Cloudflare (1.1.1.1)' },
                { value: 'google', label: 'Google (8.8.8.8)' },
                { value: 'adguard', label: 'AdGuard DNS' },
              ]}
            />
          </div>
        </div>
      </div>

      {/* Section: Account Integrations */}
      <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow">
        <div className="flex items-center gap-2 text-sm font-bold text-text-primary pb-2 border-b border-border">
          <Cloud className="w-4 h-4 text-accent-300" />
          <span>Account Integrations & Sync</span>
        </div>

        <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
          <div className="flex flex-col">
            <span className="text-xs font-bold text-text-primary">AniList & MyAnimeList Accounts</span>
            <span className="text-[11px] text-text-muted mt-0.5">
              Connect external accounts to synchronize watch history, watchlist, and ratings
            </span>
          </div>
          <button
            onClick={() => setAuthDialogOpen(true)}
            className="px-4 py-2 rounded-xl bg-accent-500 hover:bg-accent-700 text-white text-xs font-bold shadow transition-all"
          >
            Manage Accounts
          </button>
        </div>
      </div>

      {/* Section: Updates & About */}
      <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow">
        <div className="flex items-center gap-2 text-sm font-bold text-text-primary pb-2 border-b border-border">
          <Sparkles className="w-4 h-4 text-accent-300" />
          <span>Application & Updates</span>
        </div>

        <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-elevated border border-border">
          <div className="flex flex-col">
            <span className="text-xs font-bold text-text-primary">Anilili Linux Desktop</span>
            <span className="text-[11px] text-text-muted mt-0.5">
              Native Tauri v2 + Pure Rust Core • Version 0.1.0
            </span>
          </div>
          <button
            onClick={async () => {
              const res = await checkUpdateNow();
              if (res.data) {
                setUpdateDialogOpen(true);
              } else {
                showToast({ type: 'info', title: 'Up to date', message: 'You are on the latest version.' });
              }
            }}
            className="px-4 py-2 rounded-xl bg-bg-card hover:bg-bg-input text-accent-300 border border-border-focus text-xs font-bold transition-colors"
          >
            Check for Updates
          </button>
        </div>
      </div>
    </div>
  );
};
