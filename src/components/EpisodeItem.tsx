import React from 'react';
import { Play, Check, Download, AlertCircle, ListPlus } from 'lucide-react';
import { ShimmerImage } from './ShimmerImage';
import type { EpisodeItem as EpisodeItemType } from '../types';

interface EpisodeItemProps {
  episode: EpisodeItemType;
  animeId: number;
  animeTitle: string;
  animeCover: string | null;
  isActive?: boolean;
  isWatched?: boolean;
  progressPercent?: number; // 0..100
  isFiller?: boolean;
  viewMode?: 'compact' | 'list' | 'grid';
  onSelect: (ep: EpisodeItemType) => void;
  onDownload?: (ep: EpisodeItemType) => void;
  onAddToPlaylist?: (ep: EpisodeItemType) => void;
}

export const EpisodeItem: React.FC<EpisodeItemProps> = ({
  episode,
  isActive = false,
  isWatched = false,
  progressPercent = 0,
  isFiller = false,
  viewMode = 'list',
  onSelect,
  onDownload,
  onAddToPlaylist,
}) => {
  // Compact Mode: Single-line high-density row
  if (viewMode === 'compact') {
    return (
      <div
        onClick={() => onSelect(episode)}
        className={`group relative flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer select-none transition-all duration-200 border text-xs ${
          isActive
            ? 'bg-accent-tint border-accent-500 shadow-sm text-accent-300'
            : 'bg-bg-card border-border hover:bg-bg-elevated hover:border-border-focus text-text-primary'
        }`}
      >
        <div className="flex items-center gap-3 min-w-0 flex-1">
          <span className="w-12 font-bold flex-shrink-0 text-text-secondary">
            EP {episode.number}
          </span>
          <span className="truncate font-medium">
            {episode.title || `Episode ${episode.number}`}
          </span>
          {(isFiller || episode.filler) && (
            <span className="px-1.5 py-0.2 rounded bg-state-warning/20 text-yellow-400 text-[9px] font-bold border border-yellow-500/30 flex-shrink-0">
              Filler
            </span>
          )}
        </div>

        <div className="flex items-center gap-2 flex-shrink-0 ml-2">
          {isWatched && (
            <span className="flex items-center gap-1 text-[10px] text-state-success font-semibold">
              <Check className="w-3 h-3 stroke-[3]" />
              <span className="hidden sm:inline">Watched</span>
            </span>
          )}
          {onAddToPlaylist && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onAddToPlaylist(episode);
              }}
              title="Add to Custom Playlist"
              className="p-1 rounded text-text-muted hover:text-accent-300 hover:bg-bg-input transition-colors"
            >
              <ListPlus className="w-3.5 h-3.5" />
            </button>
          )}
          {onDownload && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onDownload(episode);
              }}
              title="Download Episode"
              className="p-1 rounded text-text-muted hover:text-white hover:bg-bg-input transition-colors"
            >
              <Download className="w-3.5 h-3.5" />
            </button>
          )}
          <div className="p-1 rounded-full bg-accent-500 text-white opacity-0 group-hover:opacity-100 transition-opacity">
            <Play className="w-3 h-3 fill-white ml-0.5" />
          </div>
        </div>
      </div>
    );
  }

  // Grid Mode: Card with thumbnail on top
  if (viewMode === 'grid') {
    return (
      <div
        onClick={() => onSelect(episode)}
        className={`group relative flex flex-col p-2 rounded-xl cursor-pointer select-none transition-all duration-200 border ${
          isActive
            ? 'bg-accent-tint border-accent-500 shadow-md shadow-accent-900/20'
            : 'bg-bg-card border-border hover:bg-bg-elevated hover:border-border-focus'
        }`}
      >
        <div className="relative w-full aspect-video rounded-lg overflow-hidden bg-bg-elevated flex-shrink-0 border border-border mb-2">
          {episode.image ? (
            <ShimmerImage
              src={episode.image}
              alt={`Episode ${episode.number}`}
              className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-xs font-bold text-text-muted bg-bg-elevated">
              EP {episode.number}
            </div>
          )}

          <div
            className={`absolute inset-0 flex items-center justify-center bg-bg-deepest/60 transition-opacity duration-200 ${
              isActive ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
            }`}
          >
            <div className="w-10 h-10 rounded-full bg-accent-500 text-white flex items-center justify-center shadow-lg">
              <Play className="w-5 h-5 fill-white ml-0.5" />
            </div>
          </div>

          {isWatched && !isActive && (
            <div className="absolute top-1.5 right-1.5 p-0.5 rounded-full bg-state-success text-white shadow">
              <Check className="w-2.5 h-2.5 stroke-[3]" />
            </div>
          )}

          {progressPercent > 0 && (
            <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/20">
              <div
                className="h-full bg-accent-500"
                style={{ width: `${Math.min(100, Math.max(0, progressPercent))}%` }}
              />
            </div>
          )}
        </div>

        <div className="flex items-center justify-between gap-1">
          <span
            className={`text-xs font-bold truncate ${
              isActive ? 'text-accent-300' : 'text-text-primary'
            }`}
          >
            Episode {episode.number}
          </span>
          {(isFiller || episode.filler) && (
            <span className="px-1.5 py-0.5 text-[8px] font-bold rounded bg-state-warning/20 text-yellow-400 border border-yellow-500/30">
              Filler
            </span>
          )}
        </div>

        <p className="text-[11px] text-text-muted truncate mt-0.5">
          {episode.title || `Episode ${episode.number}`}
        </p>
      </div>
    );
  }

  // Default: List Mode (16:9 thumbnail with horizontal layout)
  return (
    <div
      onClick={() => onSelect(episode)}
      className={`group relative flex items-center gap-3 p-2.5 rounded-xl cursor-pointer select-none transition-all duration-200 border ${
        isActive
          ? 'bg-accent-tint border-accent-500 shadow-md shadow-accent-900/20'
          : 'bg-bg-card border-border hover:bg-bg-elevated/70 hover:border-border-focus'
      }`}
    >
      {/* Thumbnail or Number Box */}
      <div className="relative w-24 sm:w-28 aspect-video rounded-lg overflow-hidden bg-bg-elevated flex-shrink-0 border border-border">
        {episode.image ? (
          <ShimmerImage
            src={episode.image}
            alt={`Episode ${episode.number}`}
            className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-xs font-bold text-text-muted bg-bg-elevated">
            EP {episode.number}
          </div>
        )}

        {/* Play Icon on hover or active */}
        <div
          className={`absolute inset-0 flex items-center justify-center bg-bg-deepest/60 transition-opacity duration-200 ${
            isActive ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
          }`}
        >
          <Play
            className={`w-6 h-6 fill-current ${
              isActive ? 'text-accent-300' : 'text-white'
            }`}
          />
        </div>

        {/* Watched Checkmark */}
        {isWatched && !isActive && (
          <div className="absolute top-1 right-1 p-0.5 rounded-full bg-state-success text-white shadow">
            <Check className="w-2.5 h-2.5 stroke-[3]" />
          </div>
        )}

        {/* Progress bar */}
        {progressPercent > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/20">
            <div
              className="h-full bg-accent-500"
              style={{ width: `${Math.min(100, Math.max(0, progressPercent))}%` }}
            />
          </div>
        )}
      </div>

      {/* Episode Details */}
      <div className="flex flex-col flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span
            className={`text-sm font-bold truncate ${
              isActive ? 'text-accent-300' : 'text-text-primary'
            }`}
          >
            Episode {episode.number}
          </span>
          {(isFiller || episode.filler) && (
            <span className="px-1.5 py-0.5 text-[9px] font-bold rounded bg-state-warning/20 text-yellow-400 border border-yellow-500/30">
              Filler
            </span>
          )}
        </div>

        <p className="text-xs text-text-muted truncate mt-0.5">
          {episode.title || `Episode ${episode.number}`}
        </p>

        {episode.synopsis && (
          <p className="text-[11px] text-text-secondary line-clamp-1 mt-1">
            {episode.synopsis}
          </p>
        )}
      </div>

      {/* Actions buttons on hover */}
      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
        {onAddToPlaylist && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onAddToPlaylist(episode);
            }}
            title="Add to Custom Playlist"
            className="p-2 rounded-lg text-text-muted hover:text-accent-300 hover:bg-bg-input transition-all duration-200"
          >
            <ListPlus className="w-4 h-4" />
          </button>
        )}
        {onDownload && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onDownload(episode);
            }}
            title="Download Episode"
            className="p-2 rounded-lg text-text-muted hover:text-white hover:bg-bg-input transition-all duration-200"
          >
            <Download className="w-4 h-4" />
          </button>
        )}
      </div>
    </div>
  );
};
