import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Bookmark,
  History,
  Download,
  Cloud,
  Trash2,
  Play,
  CheckCircle,
  Clock,
  AlertCircle,
  ExternalLink,
  Film,
  FolderOpen,
  HardDrive,
  ListOrdered,
} from 'lucide-react';
import {
  useWatchlist,
  useContinueWatching,
  useAllDownloads,
  useDeleteDownload,
  useSettings,
  useAniListViewer,
  useAniListLibrary,
  useSaveAniListEntry,
  usePlaylists,
} from '../hooks';
import { AnimeCard } from '../components/AnimeCard';
import { ShimmerImage } from '../components/ShimmerImage';
import { PlaylistsTab } from '../components/PlaylistsTab';
import { useUIStore } from '../stores/uiStore';
import { invoke } from '@tauri-apps/api/core';
import type { Media, DownloadRecord, LocalAnimeFile } from '../types';

export const Library: React.FC = () => {
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<'watchlist' | 'history' | 'playlists' | 'downloads' | 'anilist' | 'local'>(
    'watchlist'
  );

  const { data: watchlist = [] } = useWatchlist();
  const { data: history = [] } = useContinueWatching();
  const { data: downloads = [] } = useAllDownloads();
  const { data: playlists = [] } = usePlaylists();
  const deleteDownloadMutation = useDeleteDownload();

  const { data: settings } = useSettings();
  const { setAuthDialogOpen, showToast } = useUIStore();

  const anilistToken = settings?.anilist_token || null;
  const { data: viewer } = useAniListViewer(anilistToken);
  const [remoteStatus, setRemoteStatus] = useState<string>('CURRENT');
  const { data: remoteEntries = [] } = useAniListLibrary(
    anilistToken,
    viewer?.id || null,
    remoteStatus
  );

  // Local media scanner state
  const [localDir, setLocalDir] = useState<string>('/home');
  const [localFiles, setLocalFiles] = useState<LocalAnimeFile[]>([]);
  const [isScanning, setIsScanning] = useState<boolean>(false);

  const handleScanDirectory = async () => {
    if (!localDir) return;
    setIsScanning(true);
    try {
      const files = await invoke<LocalAnimeFile[]>('scan_local_anime_directory', { dirPath: localDir });
      setLocalFiles(files);
      showToast({ type: 'success', title: `Found ${files.length} local anime video files` });
    } catch (err: any) {
      showToast({ type: 'error', title: `Failed to scan directory: ${err}` });
    } finally {
      setIsScanning(false);
    }
  };

  const handlePlayLocalFile = async (filePath: string) => {
    try {
      await invoke('open_file_in_player', { filePath, player: settings?.external_player || 'mpv' });
    } catch (err: any) {
      showToast({ type: 'error', title: `Failed to launch player: ${err}` });
    }
  };

  return (
    <div className="flex flex-col gap-6 pb-16 animate-fade-in select-none">
      {/* Header & Tabs */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-4 rounded-2xl bg-bg-card border border-border shadow-lg">
        <div className="flex flex-wrap items-center gap-1.5 p-1 rounded-xl bg-bg-elevated border border-border">
          <button
            onClick={() => setActiveTab('watchlist')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all duration-200 ${
              activeTab === 'watchlist'
                ? 'bg-accent-500 text-white shadow-md shadow-accent-900/30'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <Bookmark className="w-3.5 h-3.5" />
            <span>Watchlist ({watchlist.length})</span>
          </button>

          <button
            onClick={() => setActiveTab('history')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all duration-200 ${
              activeTab === 'history'
                ? 'bg-accent-500 text-white shadow-md shadow-accent-900/30'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <History className="w-3.5 h-3.5" />
            <span>History ({history.length})</span>
          </button>

          <button
            onClick={() => setActiveTab('playlists')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all duration-200 cursor-pointer ${
              activeTab === 'playlists'
                ? 'bg-accent-500 text-white shadow-md shadow-accent-900/30'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <ListOrdered className="w-3.5 h-3.5" />
            <span>Playlists ({playlists.length})</span>
          </button>

          <button
            onClick={() => setActiveTab('downloads')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all duration-200 cursor-pointer ${
              activeTab === 'downloads'
                ? 'bg-accent-500 text-white shadow-md shadow-accent-900/30'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <Download className="w-3.5 h-3.5" />
            <span>Downloads ({downloads.length})</span>
          </button>

          <button
            onClick={() => setActiveTab('anilist')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all duration-200 cursor-pointer ${
              activeTab === 'anilist'
                ? 'bg-accent-500 text-white shadow-md shadow-accent-900/30'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <Cloud className="w-3.5 h-3.5" />
            <span>AniList Sync</span>
          </button>

          <button
            onClick={() => setActiveTab('local')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all duration-200 cursor-pointer ${
              activeTab === 'local'
                ? 'bg-accent-500 text-white shadow-md shadow-accent-900/30'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <HardDrive className="w-3.5 h-3.5" />
            <span>Local Files ({localFiles.length})</span>
          </button>
        </div>

        {/* User Status / Connect Account */}
        <div className="flex items-center gap-2">
          {anilistToken && viewer ? (
            <div className="flex items-center gap-2.5 px-3 py-1.5 rounded-xl bg-bg-elevated border border-border">
              {viewer.avatar_url && (
                <img
                  src={viewer.avatar_url}
                  alt={viewer.name}
                  className="w-6 h-6 rounded-full object-cover"
                />
              )}
              <span className="text-xs font-semibold text-text-primary">{viewer.name}</span>
            </div>
          ) : (
            <button
              onClick={() => setAuthDialogOpen(true)}
              className="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-accent-tint text-accent-300 border border-border-focus text-xs font-semibold hover:bg-accent-500 hover:text-white transition-colors"
            >
              <Cloud className="w-3.5 h-3.5" />
              <span>Connect AniList</span>
            </button>
          )}
        </div>
      </div>

      {/* Tab: Watchlist */}
      {activeTab === 'watchlist' && (
        <div>
          {watchlist.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 bg-bg-card rounded-2xl border border-border text-center">
              <Bookmark className="w-10 h-10 text-text-muted mb-3 opacity-30" />
              <h3 className="text-base font-bold text-text-primary">Your Watchlist is Empty</h3>
              <p className="text-xs text-text-muted mt-1">
                Browse anime from Home or Discover and click the bookmark icon to save titles here.
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
              {watchlist.map((item) => {
                const mediaMock: Media = {
                  id: item.anilist_id,
                  id_mal: null,
                  title: {
                    english: item.title,
                    romaji: item.title,
                    native: null,
                    user_preferred: item.title,
                  },
                  cover_image: {
                    extra_large: item.cover,
                    large: item.cover,
                    color: null,
                  },
                  banner_image: null,
                  description: null,
                  format: item.format,
                  status: null,
                  episodes: null,
                  duration: null,
                  season: null,
                  season_year: null,
                  average_score: item.average_score,
                  mean_score: null,
                  popularity: null,
                  favourites: null,
                  genres: [],
                  is_adult: false,
                };
                return <AnimeCard key={item.anilist_id} media={mediaMock} />;
              })}
            </div>
          )}
        </div>
      )}

      {/* Tab: History */}
      {activeTab === 'history' && (
        <div>
          {history.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 bg-bg-card rounded-2xl border border-border text-center">
              <History className="w-10 h-10 text-text-muted mb-3 opacity-30" />
              <h3 className="text-base font-bold text-text-primary">No Watch History Yet</h3>
              <p className="text-xs text-text-muted mt-1">
                Episodes you watch will automatically be tracked and saved here.
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {history.map((hist) => {
                const pct =
                  hist.duration_ms > 0
                    ? Math.round((hist.position_ms / hist.duration_ms) * 100)
                    : 0;

                return (
                  <div
                    key={`${hist.anilist_id}-${hist.episode_number}`}
                    onClick={() =>
                      navigate(
                        `/watch/${hist.anilist_id}/${hist.episode_number}?cat=${hist.category}&prov=${encodeURIComponent(
                          hist.provider
                        )}`
                      )
                    }
                    className="group flex flex-col p-3 rounded-xl bg-bg-card border border-border hover:border-border-focus hover:bg-bg-elevated cursor-pointer transition-all duration-200 select-none shadow"
                  >
                    <div className="relative w-full aspect-video rounded-lg overflow-hidden bg-bg-elevated border border-border mb-2">
                      <ShimmerImage
                        src={hist.cover}
                        alt={hist.title}
                        className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                      />
                      <div className="absolute inset-0 bg-bg-deepest/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                        <div className="w-10 h-10 rounded-full bg-accent-500 text-white flex items-center justify-center shadow-lg">
                          <Play className="w-5 h-5 fill-white ml-0.5" />
                        </div>
                      </div>
                      {/* Progress bar */}
                      <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/20">
                        <div className="h-full bg-accent-500" style={{ width: `${pct}%` }} />
                      </div>
                    </div>

                    <h4 className="text-xs font-bold text-text-primary truncate">{hist.title}</h4>
                    <div className="flex items-center justify-between mt-1 text-[11px] text-text-muted">
                      <span>
                        Episode {hist.episode_number} ({hist.category.toUpperCase()})
                      </span>
                      <span>{pct}% watched</span>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {/* Tab: Playlists */}
      {activeTab === 'playlists' && <PlaylistsTab />}

      {/* Tab: Downloads */}
      {activeTab === 'downloads' && (
        <div className="flex flex-col gap-3">
          {downloads.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-20 bg-bg-card rounded-2xl border border-border text-center">
              <Download className="w-10 h-10 text-text-muted mb-3 opacity-30" />
              <h3 className="text-base font-bold text-text-primary">No Active Downloads</h3>
              <p className="text-xs text-text-muted mt-1">
                You can download episodes for offline viewing from the anime detail page.
              </p>
            </div>
          ) : (
            downloads.map((d: DownloadRecord) => (
              <div
                key={d.id}
                className="flex items-center justify-between p-4 rounded-xl bg-bg-card border border-border shadow"
              >
                <div className="flex items-center gap-3 min-w-0">
                  <div className="w-12 h-16 rounded-lg overflow-hidden bg-bg-elevated flex-shrink-0 border border-border">
                    <ShimmerImage src={d.series_cover} alt={d.series_title} className="w-full h-full object-cover" />
                  </div>
                  <div className="flex flex-col min-w-0">
                    <span className="text-xs font-bold text-text-primary truncate">{d.series_title}</span>
                    <span className="text-[11px] text-accent-300 font-medium">
                      Episode {d.episode_num} • {d.quality} • {d.category.toUpperCase()}
                    </span>
                    <div className="flex items-center gap-2 mt-1">
                      <span
                        className={`text-[10px] font-bold px-2 py-0.5 rounded ${
                          d.status === 'COMPLETED'
                            ? 'bg-state-success/20 text-state-success'
                            : d.status === 'DOWNLOADING'
                            ? 'bg-accent-tint text-accent-300'
                            : 'bg-bg-elevated text-text-muted'
                        }`}
                      >
                        {d.status}
                      </span>
                      {d.file_size && (
                        <span className="text-[10px] text-text-muted">
                          {(d.file_size / (1024 * 1024)).toFixed(1)} MB
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                <button
                  onClick={() => deleteDownloadMutation.mutate(d.id)}
                  aria-label="Delete download"
                  className="p-2 rounded-lg text-text-muted hover:text-state-error hover:bg-state-error/10 transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            ))
          )}
        </div>
      )}

      {/* Tab: AniList Remote */}
      {activeTab === 'anilist' && (
        <div className="flex flex-col gap-4">
          {!anilistToken ? (
            <div className="flex flex-col items-center justify-center py-20 bg-bg-card rounded-2xl border border-border text-center">
              <Cloud className="w-12 h-12 text-[#02A9FF] mb-3" />
              <h3 className="text-base font-bold text-text-primary">Connect AniList</h3>
              <p className="text-xs text-text-muted mt-1 max-w-sm">
                Log in with AniList to automatically synchronize your watch lists, episode progress, and ratings.
              </p>
              <button
                onClick={() => setAuthDialogOpen(true)}
                className="mt-4 px-5 py-2.5 rounded-xl bg-[#02A9FF] hover:bg-[#0091dc] text-white text-xs font-bold transition-all shadow-md"
              >
                Connect Account
              </button>
            </div>
          ) : (
            <div className="flex flex-col gap-4">
              {/* Status Filter Chips */}
              <div className="flex items-center gap-2 overflow-x-auto pb-1 no-scrollbar">
                {['CURRENT', 'PLANNING', 'COMPLETED', 'DROPPED', 'PAUSED'].map((st) => (
                  <button
                    key={st}
                    onClick={() => setRemoteStatus(st)}
                    className={`px-3.5 py-1.5 rounded-lg text-xs font-bold transition-all ${
                      remoteStatus === st
                        ? 'bg-accent-500 text-white shadow-md'
                        : 'bg-bg-card border border-border text-text-secondary hover:text-white'
                    }`}
                  >
                    {st}
                  </button>
                ))}
              </div>

              {/* Entries Grid */}
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
                {remoteEntries.map((entry) => {
                  const mediaMock: Media = {
                    id: entry.id,
                    id_mal: null,
                    title: {
                      english: entry.title,
                      romaji: entry.title,
                      native: null,
                      user_preferred: entry.title,
                    },
                    cover_image: {
                      extra_large: entry.cover,
                      large: entry.cover,
                      color: null,
                    },
                    banner_image: null,
                    description: null,
                    format: entry.format_str,
                    status: entry.status,
                    episodes: entry.total_episodes,
                    duration: null,
                    season: null,
                    season_year: null,
                    average_score: entry.average_score,
                    mean_score: null,
                    popularity: null,
                    favourites: null,
                    genres: [],
                    is_adult: false,
                  };
                  return (
                    <div key={entry.id} className="flex flex-col">
                      <AnimeCard media={mediaMock} />
                      <div className="flex items-center justify-between text-[10px] text-text-muted px-1 mt-1 font-medium">
                        <span>Progress: {entry.progress}/{entry.total_episodes || '?'}</span>
                        {entry.score > 0 && <span>★ {entry.score}</span>}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Local Files Tab */}
      {activeTab === 'local' && (
        <div className="flex flex-col gap-6">
          {/* Folder Scanner Header */}
          <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow-md">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-xl bg-accent-tint text-accent-300 border border-border-focus">
                <FolderOpen className="w-5 h-5" />
              </div>
              <div>
                <h2 className="text-base font-bold text-text-primary">
                  Local Anime Video Directory
                </h2>
                <p className="text-xs text-text-muted">
                  Scan local hard drive folders for downloaded .mkv / .mp4 files
                </p>
              </div>
            </div>

            <div className="flex items-center gap-2 w-full md:w-auto">
              <input
                type="text"
                value={localDir}
                onChange={(e) => setLocalDir(e.target.value)}
                placeholder="/home/username/Videos/Anime"
                className="flex-1 md:w-80 px-3.5 py-2 rounded-xl bg-bg-surface border border-border text-xs text-text-primary focus:outline-none focus:border-accent-500"
              />
              <button
                onClick={handleScanDirectory}
                disabled={isScanning}
                className="px-4 py-2 rounded-xl bg-accent-500 hover:bg-accent-600 active:scale-95 text-white text-xs font-bold shadow transition-all cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
              >
                {isScanning ? (
                  <>
                    <Clock className="w-3.5 h-3.5 animate-spin" />
                    <span>Scanning...</span>
                  </>
                ) : (
                  <>
                    <HardDrive className="w-3.5 h-3.5" />
                    <span>Scan Folder</span>
                  </>
                )}
              </button>
            </div>
          </div>

          {/* Files List */}
          {localFiles.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-12 rounded-2xl bg-bg-card/50 border border-border/60 text-center">
              <Film className="w-12 h-12 text-text-muted mb-3" />
              <h3 className="text-sm font-bold text-text-primary">No local anime scanned yet</h3>
              <p className="text-xs text-text-muted max-w-sm mt-1">
                Enter your local anime directory path above and click &quot;Scan Folder&quot; to index your offline videos.
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {localFiles.map((file, idx) => (
                <div
                  key={idx}
                  className="flex items-center justify-between p-3.5 rounded-xl bg-bg-card border border-border hover:border-accent-500/50 hover:bg-bg-elevated transition-all group shadow-sm"
                >
                  <div className="flex items-center gap-3 min-w-0 pr-2">
                    <div className="p-2.5 rounded-lg bg-bg-surface border border-border text-accent-300">
                      <Film className="w-4 h-4" />
                    </div>
                    <div className="flex flex-col min-w-0">
                      <span className="text-xs font-bold text-text-primary truncate group-hover:text-accent-300 transition-colors">
                        {file.parsed_title || file.file_name}
                      </span>
                      <div className="flex items-center gap-2 text-[11px] text-text-muted mt-0.5">
                        {file.parsed_episode !== null && (
                          <span className="px-1.5 py-0.2 rounded bg-accent-500/20 text-accent-300 font-extrabold text-[10px]">
                            EP {file.parsed_episode}
                          </span>
                        )}
                        <span>{(file.file_size / (1024 * 1024)).toFixed(1)} MB</span>
                      </div>
                    </div>
                  </div>

                  <button
                    onClick={() => handlePlayLocalFile(file.file_path)}
                    className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-accent-500/20 hover:bg-accent-500 text-accent-300 hover:text-white border border-accent-500/30 text-xs font-bold transition-all cursor-pointer flex-shrink-0"
                  >
                    <Play className="w-3.5 h-3.5 fill-current" />
                    <span>Play</span>
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
