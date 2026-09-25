import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Star, Bookmark, Play } from 'lucide-react';
import { ShimmerImage } from './ShimmerImage';
import type { Media } from '../types';
import { useIsInWatchlist, useToggleWatchlist } from '../hooks/useLibrary';

interface AnimeCardProps {
  media: Media;
  className?: string;
  showProgress?: number; // 0..100
  style?: React.CSSProperties;
}

export const AnimeCard: React.FC<AnimeCardProps> = ({
  media,
  className = '',
  showProgress,
}) => {
  const navigate = useNavigate();
  const { data: inWatchlist = false } = useIsInWatchlist(media.id);
  const toggleWatchlistMutation = useToggleWatchlist();

  const title =
    media.title.english ||
    media.title.user_preferred ||
    media.title.romaji ||
    'Untitled';

  const coverUrl =
    media.cover_image.extra_large ||
    media.cover_image.large ||
    null;

  const handleCardClick = () => {
    navigate(`/detail/${media.id}`);
  };

  const handleWatchlistClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    toggleWatchlistMutation.mutate({
      anilistId: media.id,
      title,
      cover: coverUrl,
      format: media.format,
      averageScore: media.average_score,
      inWatchlist,
    });
  };

  return (
    <div
      onClick={handleCardClick}
      className={`group relative flex flex-col flex-shrink-0 cursor-pointer select-none transition-all duration-300 transform hover:-translate-y-1.5 ${className}`}
    >
      {/* Poster Container */}
      <div className="relative w-full aspect-[2/3] rounded-xl overflow-hidden bg-bg-card border border-border shadow-md group-hover:border-border-focus group-hover:shadow-xl group-hover:shadow-accent-900/20 transition-all duration-300">
        <ShimmerImage
          src={coverUrl}
          alt={title}
          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
        />

        {/* Hover Action Overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-bg-deepest/90 via-bg-deepest/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center p-3">
          <div className="w-12 h-12 rounded-full bg-accent-500 text-white flex items-center justify-center shadow-lg shadow-accent-900/50 transform scale-75 group-hover:scale-100 transition-transform duration-200">
            <Play className="w-6 h-6 fill-white ml-0.5" />
          </div>
        </div>

        {/* Top Badges */}
        <div className="absolute top-2 left-2 right-2 flex items-center justify-between pointer-events-none">
          {media.format && (
            <span className="px-2 py-0.5 text-[10px] font-bold tracking-wider uppercase rounded bg-bg-deepest/80 text-text-primary backdrop-blur-md border border-white/10 shadow">
              {media.format}
            </span>
          )}
          <div className="flex-1" />
          <button
            onClick={handleWatchlistClick}
            aria-label={inWatchlist ? 'Remove from watchlist' : 'Add to watchlist'}
            className={`pointer-events-auto p-1.5 rounded-lg backdrop-blur-md border transition-colors duration-200 ${
              inWatchlist
                ? 'bg-accent-500 text-white border-accent-300 shadow-md'
                : 'bg-bg-deepest/70 text-text-secondary border-white/10 hover:text-white hover:bg-bg-elevated'
            }`}
          >
            <Bookmark
              className={`w-3.5 h-3.5 ${inWatchlist ? 'fill-white' : ''}`}
            />
          </button>
        </div>

        {/* Dual Ratings Tag */}
        <div className="absolute bottom-2 left-2 flex items-center gap-1 max-w-[70%] flex-wrap">
          {media.rating_imdb ? (
            <div className="flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-amber-500/90 text-black text-[10px] font-black tracking-tight backdrop-blur-md shadow-sm">
              <span>IMDb {media.rating_imdb}</span>
            </div>
          ) : media.average_score !== null && media.average_score !== undefined ? (
            <div className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-bg-deepest/85 text-text-primary text-[10px] font-semibold backdrop-blur-md border border-white/10">
              <Star className="w-2.5 h-2.5 text-yellow-400 fill-yellow-400" />
              <span>{media.average_score}%</span>
            </div>
          ) : null}

          {media.score_mal ? (
            <div className="flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-blue-600/90 text-white text-[10px] font-bold tracking-tight backdrop-blur-md shadow-sm">
              <span>MAL {media.score_mal.toFixed(2)}</span>
            </div>
          ) : null}
        </div>

        {/* Episode / Season Indicator */}
        {media.episodes && (
          <div className="absolute bottom-2 right-2 px-1.5 py-0.5 rounded bg-bg-deepest/85 text-text-muted text-[10px] font-medium backdrop-blur-md border border-white/10">
            {media.episodes} eps
          </div>
        )}

        {/* Progress Bar (for Continue Watching) */}
        {showProgress !== undefined && showProgress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/20">
            <div
              className="h-full bg-accent-500"
              style={{ width: `${Math.min(100, Math.max(0, showProgress))}%` }}
            />
          </div>
        )}
      </div>

      {/* Info Container */}
      <div className="flex flex-col mt-2 px-0.5">
        <h3
          title={title}
          className="text-xs font-semibold text-text-primary group-hover:text-accent-300 line-clamp-2 transition-colors duration-200 leading-snug"
        >
          {title}
        </h3>
        <div className="flex items-center gap-1.5 mt-1 text-[11px] text-text-muted">
          <span>{media.season_year || media.season || ''}</span>
          {media.genres && media.genres.length > 0 && (
            <>
              <span>•</span>
              <span className="truncate">{media.genres[0]}</span>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
