import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  ListOrdered,
  Plus,
  Play,
  Download,
  Trash2,
  Tv,
  ArrowLeft,
  ExternalLink,
  Film,
  Clock,
  Sparkles,
  Loader2,
  Check,
  Radio,
} from 'lucide-react';
import {
  usePlaylists,
  usePlaylistItems,
  useDeletePlaylist,
  useRemoveFromPlaylist,
  useCreatePlaylist,
  launchPlaylistExternalPlayer,
} from '../hooks/usePlaylists';
import { useSettings } from '../hooks/useSettings';
import { usePlayerStore } from '../stores/playerStore';
import { useUIStore } from '../stores/uiStore';
import { invoke } from '@tauri-apps/api/core';
import type {
  CustomPlaylist,
  CustomPlaylistItem,
  ExternalPlaylistItem,
  EpisodeItem,
  EpisodesResult,
} from '../types';

export const PlaylistsTab: React.FC = () => {
  const navigate = useNavigate();
  const { data: playlists = [], isLoading } = usePlaylists();
  const deletePlaylistMutation = useDeletePlaylist();
  const removePlaylistItemMutation = useRemoveFromPlaylist();
  const createPlaylistMutation = useCreatePlaylist();
  const { data: settings } = useSettings();
  const { setCustomQueue } = usePlayerStore();
  const { showToast } = useUIStore();

  const [selectedPlaylist, setSelectedPlaylist] = useState<CustomPlaylist | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newDesc, setNewDesc] = useState('');
  const [isResolvingStreams, setIsResolvingStreams] = useState(false);
  const [isBulkDownloading, setIsBulkDownloading] = useState(false);

  const { data: items = [], isLoading: itemsLoading } = usePlaylistItems(
    selectedPlaylist ? selectedPlaylist.id : null
  );

  // Play whole playlist in App Player
  const handlePlayInApp = (startIndex: number = 0) => {
    if (items.length === 0) {
      showToast({ type: 'warning', title: 'Playlist is empty' });
      return;
    }
    const safeIndex = Math.max(0, Math.min(items.length - 1, startIndex));
    const targetItem = items[safeIndex];

    setCustomQueue(items, safeIndex, selectedPlaylist?.name || 'Playlist');
    navigate(
      `/watch/${targetItem.anilist_id}/${targetItem.episode_num}?cat=${
        targetItem.category || 'sub'
      }`
    );
  };

  // Launch Playlist in External Player (MPV or VLC)
  const handleLaunchExternal = async (player: 'mpv' | 'vlc') => {
    if (items.length === 0) {
      showToast({ type: 'warning', title: 'Playlist is empty' });
      return;
    }

    setIsResolvingStreams(true);
    showToast({
      type: 'info',
      title: `Resolving streams for ${items.length} playlist items...`,
      message: 'This may take a few seconds',
    });

    try {
      const externalItems: ExternalPlaylistItem[] = [];

      for (const item of items) {
        try {
          // Resolve episode sources
          const epData = await invoke<EpisodesResult>('get_all_episodes', {
            anilistId: item.anilist_id,
            malId: item.mal_id,
            title: item.series_title,
          });

          const providerNames = Object.keys(epData);
          if (providerNames.length > 0) {
            const prov = providerNames[0];
            const provEpisodes =
              item.category === 'dub' ? epData[prov]?.dub : epData[prov]?.sub;
            const matchEp = provEpisodes?.find(
              (e: EpisodeItem) => Math.abs(e.number - item.episode_num) < 0.01
            );

            if (matchEp) {
              const sources = await invoke<any>('get_episode_sources', {
                provider: prov,
                pipeId: matchEp.pipe_id,
                category: item.category || 'sub',
              });

              if (sources && sources.streams && sources.streams.length > 0) {
                const stream = sources.streams[0];
                externalItems.push({
                  url: stream.url,
                  title: `${item.series_title} - Episode ${item.episode_num}`,
                  referer: stream.headers?.Referer || stream.headers?.referer || null,
                });
              }
            }
          }
        } catch (e) {
          console.warn(`Could not resolve stream for ${item.series_title} Ep ${item.episode_num}:`, e);
        }
      }

      if (externalItems.length === 0) {
        showToast({
          type: 'error',
          title: 'Failed to resolve streaming links',
          message: 'Check your internet connection or stream providers.',
        });
        return;
      }

      await launchPlaylistExternalPlayer(
        player,
        selectedPlaylist?.name || 'Anilili_Playlist',
        externalItems
      );

      showToast({
        type: 'success',
        title: `Launched ${externalItems.length} episodes in ${player.toUpperCase()}!`,
      });
    } catch (err: any) {
      showToast({
        type: 'error',
        title: `Failed to launch ${player.toUpperCase()}`,
        message: err?.message || String(err),
      });
    } finally {
      setIsResolvingStreams(false);
    }
  };

  // Bulk download all items in playlist
  const handleDownloadAll = async () => {
    if (items.length === 0) return;
    setIsBulkDownloading(true);
    showToast({
      type: 'info',
      title: `Starting batch download for ${items.length} items...`,
    });

    let queuedCount = 0;
    try {
      for (const item of items) {
        try {
          const downloadRecord = {
            id: crypto.randomUUID(),
            anilist_id: item.anilist_id,
            mal_id: item.mal_id,
            series_title: item.series_title,
            series_cover: item.series_cover,
            episode_num: item.episode_num,
            episode_title: item.episode_title,
            category: item.category || 'sub',
            provider: 'auto',
            quality: settings?.download_quality || '1080p',
            status: 'queued',
            file_path: null,
            file_size: null,
            duration_s: null,
            error_msg: null,
            created_at: Math.floor(Date.now() / 1000),
            updated_at: Math.floor(Date.now() / 1000),
          };

          await invoke('create_download', { record: downloadRecord });
          queuedCount++;
        } catch (err) {
          console.error('Failed to create download record:', err);
        }
      }

      showToast({
        type: 'success',
        title: `Queued ${queuedCount} downloads!`,
        message: 'Head over to the Downloads tab to track progress.',
      });
    } catch (err: any) {
      showToast({
        type: 'error',
        title: 'Error during batch download queue',
        message: err?.message || String(err),
      });
    } finally {
      setIsBulkDownloading(false);
    }
  };

  // Handle Delete Playlist
  const handleDeletePlaylist = async (playlistId: string, name: string) => {
    if (window.confirm(`Are you sure you want to delete playlist "${name}"?`)) {
      try {
        await deletePlaylistMutation.mutateAsync(playlistId);
        showToast({ type: 'success', title: `Deleted playlist "${name}"` });
        if (selectedPlaylist?.id === playlistId) {
          setSelectedPlaylist(null);
        }
      } catch (err: any) {
        showToast({ type: 'error', title: 'Failed to delete playlist', message: err?.message });
      }
    }
  };

  // Create playlist submission
  const handleCreatePlaylist = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTitle.trim()) return;

    try {
      const pl = await createPlaylistMutation.mutateAsync({
        name: newTitle.trim(),
        description: newDesc.trim() || undefined,
      });
      showToast({ type: 'success', title: `Created playlist "${pl.name}"` });
      setNewTitle('');
      setNewDesc('');
      setShowCreateModal(false);
      setSelectedPlaylist(pl);
    } catch (err: any) {
      showToast({ type: 'error', title: 'Failed to create playlist', message: err?.message });
    }
  };

  return (
    <div className="space-y-6">
      {/* View: Playlist Detail View */}
      {selectedPlaylist ? (
        <div className="space-y-6 animate-fade-in">
          {/* Back Navigation Bar */}
          <div className="flex items-center justify-between">
            <button
              onClick={() => setSelectedPlaylist(null)}
              className="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-bg-surface hover:bg-bg-elevated text-xs font-bold text-text-secondary hover:text-white transition-colors cursor-pointer border border-border"
            >
              <ArrowLeft className="w-4 h-4" />
              <span>Back to all Playlists</span>
            </button>

            <button
              onClick={() => handleDeletePlaylist(selectedPlaylist.id, selectedPlaylist.name)}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-red-500/10 hover:bg-red-500/20 text-xs font-bold text-red-400 hover:text-red-300 transition-colors cursor-pointer border border-red-500/20"
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span>Delete Playlist</span>
            </button>
          </div>

          {/* Playlist Hero Info Card */}
          <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-6 p-6 rounded-2xl bg-bg-card border border-border shadow-xl">
            <div className="flex items-center gap-5">
              <div className="w-24 h-24 sm:w-28 sm:h-28 rounded-2xl overflow-hidden bg-bg-elevated border border-border grid grid-cols-2 gap-0.5 shadow-md flex-shrink-0">
                {selectedPlaylist.covers && selectedPlaylist.covers.length > 0 ? (
                  selectedPlaylist.covers.slice(0, 4).map((c, i) => (
                    <img key={i} src={c} alt="Cover" className="w-full h-full object-cover" />
                  ))
                ) : (
                  <div className="col-span-2 row-span-2 flex items-center justify-center">
                    <Tv className="w-8 h-8 text-text-muted" />
                  </div>
                )}
              </div>

              <div>
                <span className="text-[11px] font-extrabold uppercase tracking-widest text-accent-400">
                  Custom Playlist
                </span>
                <h2 className="text-xl sm:text-2xl font-black text-white mt-1">
                  {selectedPlaylist.name}
                </h2>
                {selectedPlaylist.description && (
                  <p className="text-xs text-text-muted mt-1 max-w-lg">
                    {selectedPlaylist.description}
                  </p>
                )}
                <div className="flex items-center gap-3 text-xs text-text-secondary mt-3">
                  <span className="font-semibold">{items.length} Episodes</span>
                  <span>•</span>
                  <span>Created {new Date(selectedPlaylist.created_at * 1000).toLocaleDateString()}</span>
                </div>
              </div>
            </div>

            {/* Actions Bar */}
            <div className="flex flex-wrap items-center gap-2.5 w-full md:w-auto">
              <button
                onClick={() => handlePlayInApp(0)}
                disabled={items.length === 0}
                className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-accent-500 hover:bg-accent-600 disabled:opacity-50 text-white font-extrabold text-xs shadow-lg shadow-accent-900/40 active:scale-95 transition-all cursor-pointer"
              >
                <Play className="w-4 h-4 fill-white" />
                <span>Play in App</span>
              </button>

              <button
                onClick={() => handleLaunchExternal('mpv')}
                disabled={items.length === 0 || isResolvingStreams}
                className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-bg-surface hover:bg-bg-elevated border border-border text-text-primary hover:text-white font-bold text-xs active:scale-95 transition-all cursor-pointer"
                title="Launch in MPV Player with full playlist"
              >
                {isResolvingStreams ? (
                  <Loader2 className="w-4 h-4 animate-spin text-accent-400" />
                ) : (
                  <Tv className="w-4 h-4 text-accent-400" />
                )}
                <span>Play in MPV</span>
              </button>

              <button
                onClick={() => handleLaunchExternal('vlc')}
                disabled={items.length === 0 || isResolvingStreams}
                className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-bg-surface hover:bg-bg-elevated border border-border text-text-primary hover:text-white font-bold text-xs active:scale-95 transition-all cursor-pointer"
                title="Launch in VLC Player with full playlist"
              >
                <Radio className="w-4 h-4 text-orange-400" />
                <span>Play in VLC</span>
              </button>

              <button
                onClick={handleDownloadAll}
                disabled={items.length === 0 || isBulkDownloading}
                className="flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-bg-surface hover:bg-bg-elevated border border-border text-text-primary hover:text-white font-bold text-xs active:scale-95 transition-all cursor-pointer"
                title="Download all episodes in this playlist"
              >
                {isBulkDownloading ? (
                  <Loader2 className="w-4 h-4 animate-spin text-accent-400" />
                ) : (
                  <Download className="w-4 h-4 text-state-success" />
                )}
                <span>Download All</span>
              </button>
            </div>
          </div>

          {/* Playlist Items List */}
          <div className="space-y-3">
            <h3 className="text-sm font-bold text-text-secondary uppercase tracking-wider">
              Episodes in Playlist ({items.length})
            </h3>

            {itemsLoading ? (
              <div className="flex justify-center p-12">
                <Loader2 className="w-8 h-8 animate-spin text-accent-400" />
              </div>
            ) : items.length === 0 ? (
              <div className="text-center p-12 bg-bg-card border border-border rounded-2xl">
                <Tv className="w-10 h-10 text-text-muted mx-auto mb-2" />
                <h4 className="text-sm font-bold text-text-primary">This playlist has no episodes</h4>
                <p className="text-xs text-text-muted mt-1">
                  Navigate to any anime detail page and click the "Add to Playlist" button on an episode to populate it.
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                {items.map((item, index) => (
                  <div
                    key={item.id}
                    className="flex items-center justify-between p-3 rounded-xl bg-bg-card border border-border hover:border-accent-500/50 hover:bg-bg-elevated transition-all group"
                  >
                    <div className="flex items-center gap-3.5 min-w-0 pr-3">
                      <span className="w-6 text-center text-xs font-bold text-text-muted">
                        {index + 1}
                      </span>
                      <div className="w-12 h-16 rounded-lg overflow-hidden bg-bg-elevated border border-border flex-shrink-0">
                        {item.series_cover ? (
                          <img
                            src={item.series_cover}
                            alt={item.series_title}
                            className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                          />
                        ) : (
                          <div className="w-full h-full flex items-center justify-center text-[10px] text-text-muted">
                            Cover
                          </div>
                        )}
                      </div>

                      <div className="min-w-0">
                        <div className="flex items-center gap-2">
                          <h4 className="text-xs sm:text-sm font-bold text-text-primary group-hover:text-accent-300 truncate">
                            {item.series_title}
                          </h4>
                          <span className="px-1.5 py-0.2 rounded bg-accent-500/10 text-accent-300 border border-accent-500/20 text-[10px] font-extrabold flex-shrink-0">
                            {item.category.toUpperCase()}
                          </span>
                        </div>
                        <p className="text-xs text-text-muted mt-0.5 truncate">
                          Episode {item.episode_num} {item.episode_title ? `• ${item.episode_title}` : ''}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center gap-2 flex-shrink-0">
                      <button
                        onClick={() => handlePlayInApp(index)}
                        className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-accent-500/20 hover:bg-accent-500 text-accent-300 hover:text-white border border-accent-500/30 text-xs font-bold transition-all cursor-pointer"
                        title="Play starting here"
                      >
                        <Play className="w-3.5 h-3.5 fill-current" />
                        <span>Play</span>
                      </button>

                      <button
                        onClick={async () => {
                          await removePlaylistItemMutation.mutateAsync({
                            itemId: item.id,
                            playlistId: selectedPlaylist.id,
                          });
                          showToast({ type: 'info', title: 'Removed item from playlist' });
                        }}
                        className="p-1.5 rounded-lg text-text-muted hover:text-red-400 hover:bg-red-500/10 transition-colors"
                        title="Remove from playlist"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      ) : (
        /* View: All Playlists Grid */
        <div className="space-y-6 animate-fade-in">
          {/* Action Header */}
          <div className="flex items-center justify-between p-4 rounded-2xl bg-bg-card border border-border shadow-sm">
            <div>
              <h2 className="text-base font-bold text-text-primary">
                Custom Playlists & Queues
              </h2>
              <p className="text-xs text-text-muted mt-0.5">
                Curate cross-anime watchlists to binge sequentially, batch download, or stream in MPV/VLC.
              </p>
            </div>

            <button
              onClick={() => setShowCreateModal(true)}
              className="flex items-center gap-2 px-4 py-2 rounded-xl bg-accent-500 hover:bg-accent-600 active:scale-95 text-white font-bold text-xs shadow-md shadow-accent-900/30 transition-all cursor-pointer"
            >
              <Plus className="w-4 h-4" />
              <span>New Playlist</span>
            </button>
          </div>

          {/* Playlists Grid */}
          {isLoading ? (
            <div className="flex justify-center p-12">
              <Loader2 className="w-8 h-8 animate-spin text-accent-400" />
            </div>
          ) : playlists.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-16 rounded-2xl bg-bg-card border border-border text-center">
              <ListOrdered className="w-12 h-12 text-text-muted mb-3" />
              <h3 className="text-sm font-bold text-text-primary">No playlists yet</h3>
              <p className="text-xs text-text-muted max-w-sm mt-1 mb-4">
                Create a custom playlist to bundle episodes across multiple anime into a continuous queue for playback or downloading.
              </p>
              <button
                onClick={() => setShowCreateModal(true)}
                className="flex items-center gap-2 px-4 py-2 rounded-xl bg-accent-500 hover:bg-accent-600 text-white font-bold text-xs shadow transition-all cursor-pointer"
              >
                <Plus className="w-4 h-4" />
                <span>Create Your First Playlist</span>
              </button>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {playlists.map((playlist) => (
                <div
                  key={playlist.id}
                  onClick={() => setSelectedPlaylist(playlist)}
                  className="group flex flex-col p-3.5 rounded-2xl bg-bg-card border border-border hover:border-accent-500 hover:bg-bg-elevated transition-all duration-300 shadow-sm hover:shadow-xl cursor-pointer"
                >
                  {/* Collage Thumbnail */}
                  <div className="relative w-full aspect-video rounded-xl overflow-hidden bg-bg-surface border border-border mb-3 grid grid-cols-2 gap-0.5">
                    {playlist.covers && playlist.covers.length > 0 ? (
                      playlist.covers.slice(0, 4).map((c, i) => (
                        <img
                          key={i}
                          src={c}
                          alt="Cover"
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                      ))
                    ) : (
                      <div className="col-span-2 row-span-2 flex items-center justify-center">
                        <Tv className="w-8 h-8 text-text-muted" />
                      </div>
                    )}

                    <div className="absolute bottom-2 right-2 px-2 py-0.5 rounded-md bg-black/75 backdrop-blur-xs text-[10px] font-extrabold text-white border border-white/10">
                      {playlist.item_count} ep{playlist.item_count !== 1 ? 's' : ''}
                    </div>
                  </div>

                  {/* Title & Desc */}
                  <h4 className="text-sm font-bold text-text-primary group-hover:text-accent-300 transition-colors truncate">
                    {playlist.name}
                  </h4>
                  <p className="text-[11px] text-text-muted truncate mt-0.5">
                    {playlist.description || 'No description provided.'}
                  </p>

                  <div className="flex items-center justify-between mt-4 pt-3 border-t border-border/50 text-[11px] text-text-muted">
                    <span>{new Date(playlist.created_at * 1000).toLocaleDateString()}</span>
                    <span className="text-accent-400 font-bold group-hover:translate-x-1 transition-transform">
                      View details ›
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Create Playlist Modal */}
      {showCreateModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-xs animate-fade-in"
          onClick={() => setShowCreateModal(false)}
        >
          <div
            className="w-full max-w-md bg-bg-card border border-border rounded-2xl shadow-2xl p-5 space-y-4 animate-scale-up"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between border-b border-border pb-3">
              <div className="flex items-center gap-2">
                <ListOrdered className="w-5 h-5 text-accent-400" />
                <h3 className="text-base font-bold text-white">Create Custom Playlist</h3>
              </div>
              <button
                onClick={() => setShowCreateModal(false)}
                className="text-xs text-text-muted hover:text-white"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleCreatePlaylist} className="space-y-3.5">
              <div>
                <label className="text-xs font-semibold text-text-secondary block mb-1">
                  Playlist Name *
                </label>
                <input
                  type="text"
                  placeholder="e.g., Best Shonen Fights, Weekend Marathon..."
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  autoFocus
                  required
                  className="w-full px-3 py-2 text-sm bg-bg-surface border border-border rounded-xl text-white focus:outline-none focus:border-accent-500"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-text-secondary block mb-1">
                  Description (optional)
                </label>
                <textarea
                  rows={3}
                  placeholder="What is this playlist about?"
                  value={newDesc}
                  onChange={(e) => setNewDesc(e.target.value)}
                  className="w-full px-3 py-2 text-xs bg-bg-surface border border-border rounded-xl text-white focus:outline-none focus:border-accent-500 resize-none"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-text-muted hover:text-white hover:bg-bg-elevated transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={!newTitle.trim() || createPlaylistMutation.isPending}
                  className="px-5 py-2 rounded-xl bg-accent-500 hover:bg-accent-600 disabled:opacity-50 text-white font-bold text-xs shadow-md transition-all cursor-pointer"
                >
                  {createPlaylistMutation.isPending ? 'Creating...' : 'Create Playlist'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
