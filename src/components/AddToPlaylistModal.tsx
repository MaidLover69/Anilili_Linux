import React, { useState } from 'react';
import {
  X,
  Plus,
  ListPlus,
  FolderPlus,
  Check,
  Music,
  Tv,
  Loader2,
} from 'lucide-react';
import {
  usePlaylists,
  useCreatePlaylist,
  useAddMultipleToPlaylist,
} from '../hooks/usePlaylists';
import { useUIStore } from '../stores/uiStore';
import type { EpisodeItem, CustomPlaylistItem } from '../types';

interface AddToPlaylistModalProps {
  isOpen: boolean;
  onClose: () => void;
  anilistId: number;
  malId?: number | null;
  seriesTitle: string;
  seriesCover: string | null;
  category?: 'sub' | 'dub';
  episodes: EpisodeItem[]; // can be 1 or multiple episodes
}

export const AddToPlaylistModal: React.FC<AddToPlaylistModalProps> = ({
  isOpen,
  onClose,
  anilistId,
  malId = null,
  seriesTitle,
  seriesCover,
  category = 'sub',
  episodes,
}) => {
  const [newPlaylistName, setNewPlaylistName] = useState('');
  const [newPlaylistDesc, setNewPlaylistDesc] = useState('');
  const [isCreatingNew, setIsCreatingNew] = useState(false);
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<string | null>(null);

  const { data: playlists = [], isLoading } = usePlaylists();
  const createPlaylistMutation = useCreatePlaylist();
  const addItemsMutation = useAddMultipleToPlaylist();
  const { showToast } = useUIStore();

  if (!isOpen) return null;

  const handleAddEpisodesToPlaylist = async (playlistId: string, playlistTitle: string) => {
    try {
      const now = Math.floor(Date.now() / 1000);
      const items: CustomPlaylistItem[] = episodes.map((ep, idx) => ({
        id: crypto.randomUUID(),
        playlist_id: playlistId,
        anilist_id: anilistId,
        mal_id: malId ?? null,
        episode_num: ep.number,
        episode_title: ep.title || null,
        series_title: seriesTitle,
        series_cover: seriesCover,
        category,
        sort_order: idx,
        added_at: now,
      }));

      await addItemsMutation.mutateAsync({ playlistId, items });
      showToast({
        type: 'success',
        title: `Added ${episodes.length} episode${episodes.length > 1 ? 's' : ''} to "${playlistTitle}"`,
      });
      onClose();
    } catch (err: any) {
      showToast({
        type: 'error',
        title: 'Failed to add to playlist',
        message: err?.message || String(err),
      });
    }
  };

  const handleCreateAndAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newPlaylistName.trim()) return;

    try {
      const newPlaylist = await createPlaylistMutation.mutateAsync({
        name: newPlaylistName.trim(),
        description: newPlaylistDesc.trim() || undefined,
      });

      await handleAddEpisodesToPlaylist(newPlaylist.id, newPlaylist.name);
    } catch (err: any) {
      showToast({
        type: 'error',
        title: 'Failed to create playlist',
        message: err?.message || String(err),
      });
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-xs animate-fade-in"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md bg-bg-card border border-border rounded-2xl shadow-2xl overflow-hidden animate-scale-up"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border bg-bg-surface/50">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-accent-500/10 border border-accent-500/20 text-accent-400">
              <ListPlus className="w-5 h-5" />
            </div>
            <div>
              <h3 className="text-base font-bold text-text-primary">
                Add to Custom Playlist
              </h3>
              <p className="text-xs text-text-muted truncate max-w-xs">
                {episodes.length === 1
                  ? `${seriesTitle} • Ep ${episodes[0].number}`
                  : `${episodes.length} selected episodes from ${seriesTitle}`}
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-text-muted hover:text-white hover:bg-white/10 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4 max-h-[60vh] overflow-y-auto">
          {/* Create new toggle */}
          {!isCreatingNew ? (
            <button
              onClick={() => setIsCreatingNew(true)}
              className="w-full flex items-center justify-center gap-2 p-3 rounded-xl border border-dashed border-accent-500/40 hover:border-accent-500 bg-accent-500/5 hover:bg-accent-500/10 text-accent-300 font-semibold text-sm transition-all cursor-pointer"
            >
              <Plus className="w-4 h-4" />
              <span>Create New Playlist</span>
            </button>
          ) : (
            <form onSubmit={handleCreateAndAdd} className="p-3.5 rounded-xl bg-bg-surface border border-accent-500/30 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-accent-400 uppercase tracking-wider">
                  New Playlist
                </span>
                <button
                  type="button"
                  onClick={() => setIsCreatingNew(false)}
                  className="text-xs text-text-muted hover:text-white"
                >
                  Cancel
                </button>
              </div>
              <input
                type="text"
                placeholder="Playlist name (e.g., Epic Fights, Road Trip)..."
                value={newPlaylistName}
                onChange={(e) => setNewPlaylistName(e.target.value)}
                autoFocus
                className="w-full px-3 py-2 text-sm bg-bg-card border border-border rounded-lg text-white focus:outline-none focus:border-accent-500"
              />
              <input
                type="text"
                placeholder="Description (optional)..."
                value={newPlaylistDesc}
                onChange={(e) => setNewPlaylistDesc(e.target.value)}
                className="w-full px-3 py-1.5 text-xs bg-bg-card border border-border rounded-lg text-white focus:outline-none focus:border-accent-500"
              />
              <button
                type="submit"
                disabled={!newPlaylistName.trim() || createPlaylistMutation.isPending}
                className="w-full py-2 px-3 bg-accent-500 hover:bg-accent-600 disabled:opacity-50 text-white rounded-lg text-xs font-bold flex items-center justify-center gap-1.5 transition-colors cursor-pointer"
              >
                {createPlaylistMutation.isPending ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <>
                    <Check className="w-4 h-4" />
                    <span>Create & Add Episodes</span>
                  </>
                )}
              </button>
            </form>
          )}

          {/* Existing Playlists */}
          <div className="space-y-2">
            <h4 className="text-xs font-bold uppercase tracking-wider text-text-muted">
              Existing Playlists ({playlists.length})
            </h4>

            {isLoading ? (
              <div className="flex justify-center p-6">
                <Loader2 className="w-6 h-6 animate-spin text-accent-400" />
              </div>
            ) : playlists.length === 0 ? (
              <div className="text-center p-6 border border-border rounded-xl bg-bg-surface/30">
                <p className="text-sm text-text-muted">No custom playlists created yet.</p>
                <p className="text-xs text-text-secondary mt-1">
                  Create your first playlist above to get started!
                </p>
              </div>
            ) : (
              <div className="space-y-2 max-h-56 overflow-y-auto pr-1">
                {playlists.map((playlist) => (
                  <div
                    key={playlist.id}
                    onClick={() => handleAddEpisodesToPlaylist(playlist.id, playlist.name)}
                    className="flex items-center justify-between p-3 rounded-xl border border-border hover:border-accent-500 bg-bg-surface hover:bg-bg-elevated transition-all cursor-pointer group"
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <div className="w-10 h-10 rounded-lg overflow-hidden bg-bg-elevated border border-border flex items-center justify-center flex-shrink-0">
                        {playlist.covers && playlist.covers[0] ? (
                          <img
                            src={playlist.covers[0]}
                            alt={playlist.name}
                            className="w-full h-full object-cover"
                          />
                        ) : (
                          <Tv className="w-5 h-5 text-text-muted" />
                        )}
                      </div>
                      <div className="min-w-0">
                        <h5 className="text-sm font-semibold text-text-primary group-hover:text-accent-300 truncate">
                          {playlist.name}
                        </h5>
                        <p className="text-[11px] text-text-muted">
                          {playlist.item_count} episode{playlist.item_count !== 1 ? 's' : ''}
                          {playlist.description && ` • ${playlist.description}`}
                        </p>
                      </div>
                    </div>

                    <button
                      type="button"
                      disabled={addItemsMutation.isPending}
                      className="px-3 py-1.5 rounded-lg bg-accent-500/10 hover:bg-accent-500 text-accent-300 hover:text-white border border-accent-500/30 text-xs font-bold transition-all flex items-center gap-1 flex-shrink-0"
                    >
                      <Plus className="w-3.5 h-3.5" />
                      <span>Add</span>
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="p-3 border-t border-border bg-bg-surface/30 flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-text-secondary hover:text-white hover:bg-bg-elevated transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
