import React, { useState, useEffect, useRef } from 'react';
import {
  X,
  Play,
  CheckSquare,
  Square,
  Search,
  ListOrdered,
  Sparkles,
  Layers,
  Trash2,
} from 'lucide-react';
import type { EpisodeItem } from '../types';

interface PlayerPlaylistDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  episodes: EpisodeItem[];
  currentEpisodeNumber: number;
  onSelectEpisode: (epNum: number) => void;
  queuedEpisodes: number[];
  onToggleQueueEpisode: (epNum: number) => void;
  onPlayQueue: () => void;
  onClearQueue: () => void;
}

export const PlayerPlaylistDrawer: React.FC<PlayerPlaylistDrawerProps> = ({
  isOpen,
  onClose,
  episodes,
  currentEpisodeNumber,
  onSelectEpisode,
  queuedEpisodes,
  onToggleQueueEpisode,
  onPlayQueue,
  onClearQueue,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [multiSelectMode, setMultiSelectMode] = useState(false);
  const activeEpRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (isOpen && activeEpRef.current) {
      activeEpRef.current.scrollIntoView({ block: 'center', behavior: 'smooth' });
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const filteredEpisodes = episodes.filter((ep) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      ep.number.toString().includes(q) ||
      (ep.title && ep.title.toLowerCase().includes(q))
    );
  });

  return (
    <div
      className="fixed inset-0 z-50 flex justify-end bg-black/60 backdrop-blur-xs animate-fade-in"
      onClick={onClose}
    >
      <div
        className="w-full max-w-sm h-full bg-bg-card/95 border-l border-border flex flex-col shadow-2xl overflow-hidden animate-slide-left"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Drawer Header */}
        <div className="flex items-center justify-between p-4 border-b border-border bg-bg-surface/60">
          <div className="flex items-center gap-2">
            <ListOrdered className="w-5 h-5 text-accent-400" />
            <div>
              <h3 className="text-sm font-bold text-text-primary">Episode Playlist</h3>
              <p className="text-[11px] text-text-muted">
                {episodes.length} Episodes available
              </p>
            </div>
          </div>

          <div className="flex items-center gap-1.5">
            <button
              onClick={() => setMultiSelectMode(!multiSelectMode)}
              className={`p-1.5 rounded-lg border text-xs font-bold transition-all cursor-pointer ${
                multiSelectMode
                  ? 'bg-accent-500 text-white border-accent-400'
                  : 'bg-bg-surface text-text-secondary border-border hover:text-white'
              }`}
              title="Toggle Multi-Select Playback Queue"
            >
              <Layers className="w-4 h-4" />
            </button>
            <button
              onClick={onClose}
              className="p-1.5 rounded-lg text-text-secondary hover:text-white hover:bg-white/10 transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Multi-Select Action Bar (if active) */}
        {multiSelectMode && (
          <div className="flex items-center justify-between p-2.5 bg-accent-500/10 border-b border-accent-500/20 px-4">
            <span className="text-xs font-bold text-accent-300">
              {queuedEpisodes.length} selected for Queue
            </span>
            <div className="flex items-center gap-1.5">
              {queuedEpisodes.length > 0 && (
                <>
                  <button
                    onClick={onClearQueue}
                    className="p-1 rounded bg-bg-surface hover:bg-red-500/20 text-red-300 border border-border text-[11px] font-bold transition-colors"
                    title="Clear selected queue"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => {
                      onPlayQueue();
                      onClose();
                    }}
                    className="px-3 py-1 rounded bg-accent-500 hover:bg-accent-600 text-white text-xs font-bold shadow transition-all cursor-pointer flex items-center gap-1"
                  >
                    <Play className="w-3 h-3 fill-current" />
                    <span>Play Queue</span>
                  </button>
                </>
              )}
            </div>
          </div>
        )}

        {/* Search Bar */}
        <div className="p-3 border-b border-border bg-bg-surface/30">
          <div className="relative">
            <Search className="absolute left-3 top-2.5 w-3.5 h-3.5 text-text-muted" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search episode number or title..."
              className="w-full pl-9 pr-3 py-1.5 rounded-xl bg-bg-surface border border-border text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-500"
            />
          </div>
        </div>

        {/* Episode Items List */}
        <div className="flex-1 overflow-y-auto p-3 flex flex-col gap-2">
          {filteredEpisodes.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-8 text-center text-text-muted text-xs">
              No episodes match your search.
            </div>
          ) : (
            filteredEpisodes.map((ep) => {
              const isCurrent = ep.number === currentEpisodeNumber;
              const isQueued = queuedEpisodes.includes(ep.number);

              return (
                <div
                  key={ep.number}
                  ref={isCurrent ? activeEpRef : null}
                  className={`flex items-center justify-between p-2.5 rounded-xl border transition-all cursor-pointer group ${
                    isCurrent
                      ? 'border-accent-500 bg-accent-500/20 text-white shadow-md shadow-accent-950/30'
                      : isQueued
                      ? 'border-accent-400/50 bg-accent-500/10 text-accent-200'
                      : 'border-border bg-bg-surface/80 hover:bg-bg-elevated hover:border-border-focus text-text-secondary hover:text-white'
                  }`}
                  onClick={() => {
                    if (multiSelectMode) {
                      onToggleQueueEpisode(ep.number);
                    } else {
                      onSelectEpisode(ep.number);
                      onClose();
                    }
                  }}
                >
                  <div className="flex items-center gap-2.5 min-w-0 pr-2">
                    {multiSelectMode && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onToggleQueueEpisode(ep.number);
                        }}
                        className="text-accent-300 hover:text-accent-200 cursor-pointer"
                      >
                        {isQueued ? (
                          <CheckSquare className="w-4 h-4 text-accent-400" />
                        ) : (
                          <Square className="w-4 h-4 text-text-muted" />
                        )}
                      </button>
                    )}

                    <span
                      className={`w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black flex-shrink-0 ${
                        isCurrent
                          ? 'bg-accent-500 text-white'
                          : 'bg-bg-card border border-border text-text-muted group-hover:text-text-primary'
                      }`}
                    >
                      {ep.number}
                    </span>

                    <div className="flex flex-col min-w-0">
                      <span className="text-xs font-bold truncate">
                        {ep.title || `Episode ${ep.number}`}
                      </span>
                      {ep.filler && (
                        <span className="text-[10px] text-amber-400 font-semibold">
                          Filler Episode
                        </span>
                      )}
                    </div>
                  </div>

                  {isCurrent && (
                    <span className="px-2 py-0.5 rounded bg-accent-500 text-white text-[10px] font-extrabold flex-shrink-0">
                      Playing
                    </span>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
