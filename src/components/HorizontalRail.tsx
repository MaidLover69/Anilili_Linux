import React, { useRef } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { AnimeCard } from './AnimeCard';
import { SkeletonCard } from './SkeletonCard';
import type { Media, HistoryEntry } from '../types';
import { useNavigate } from 'react-router-dom';

interface HorizontalRailProps {
  title: string;
  subtitle?: string;
  items?: Media[];
  historyItems?: HistoryEntry[];
  isLoading?: boolean;
  viewAllLink?: string;
}

export const HorizontalRail: React.FC<HorizontalRailProps> = ({
  title,
  subtitle,
  items,
  historyItems,
  isLoading = false,
  viewAllLink,
}) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  const handleScroll = (direction: 'left' | 'right') => {
    if (!scrollRef.current) return;
    const offset = direction === 'left' ? -600 : 600;
    scrollRef.current.scrollBy({ left: offset, behavior: 'smooth' });
  };

  const hasContent = (items && items.length > 0) || (historyItems && historyItems.length > 0);
  if (!isLoading && !hasContent) return null;

  return (
    <div className="flex flex-col gap-3 my-6 group/rail">
      {/* Rail Header */}
      <div className="flex items-end justify-between px-1">
        <div className="flex flex-col">
          <h2 className="text-lg md:text-xl font-bold text-text-primary tracking-tight">
            {title}
          </h2>
          {subtitle && (
            <span className="text-xs text-text-muted mt-0.5">{subtitle}</span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {viewAllLink && (
            <button
              onClick={() => navigate(viewAllLink)}
              className="text-xs font-semibold text-accent-300 hover:text-white transition-colors mr-2"
            >
              View All
            </button>
          )}

          <button
            onClick={() => handleScroll('left')}
            aria-label="Scroll left"
            className="p-1.5 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-white hover:border-border-focus transition-all duration-200"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <button
            onClick={() => handleScroll('right')}
            aria-label="Scroll right"
            className="p-1.5 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-white hover:border-border-focus transition-all duration-200"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Scrollable Container */}
      <div
        ref={scrollRef}
        className="flex items-start gap-4 overflow-x-auto pb-4 pt-1 px-1 scroll-smooth no-scrollbar"
        style={{ scrollSnapType: 'x mandatory' }}
      >
        {isLoading ? (
          Array.from({ length: 7 }).map((_, i) => (
            <SkeletonCard
              key={i}
              className="w-[140px] sm:w-[160px] md:w-[180px] flex-shrink-0"
            />
          ))
        ) : historyItems && historyItems.length > 0 ? (
          historyItems.map((hist) => {
            const mediaMock: Media = {
              id: hist.anilist_id,
              id_mal: null,
              title: {
                english: hist.title,
                romaji: hist.title,
                native: null,
                user_preferred: hist.title,
              },
              cover_image: {
                extra_large: hist.cover,
                large: hist.cover,
                color: null,
              },
              banner_image: null,
              description: null,
              format: 'TV',
              status: null,
              episodes: null,
              duration: null,
              season: null,
              season_year: null,
              average_score: null,
              mean_score: null,
              popularity: null,
              favourites: null,
              genres: [],
              is_adult: false,
            };
            const progressPct =
              hist.duration_ms > 0
                ? (hist.position_ms / hist.duration_ms) * 100
                : 0;

            return (
              <div
                key={`${hist.anilist_id}-${hist.episode_number}`}
                className="w-[140px] sm:w-[160px] md:w-[180px] flex-shrink-0 flex flex-col"
                style={{ scrollSnapAlign: 'start' }}
              >
                <AnimeCard
                  media={mediaMock}
                  showProgress={progressPct}
                  className="w-full"
                />
                <div className="text-[11px] text-accent-300 font-medium px-1 mt-1 truncate">
                  Episode {hist.episode_number}
                  {hist.episode_title ? ` - ${hist.episode_title}` : ''}
                </div>
              </div>
            );
          })
        ) : items ? (
          items.map((anime) => (
            <AnimeCard
              key={anime.id}
              media={anime}
              className="w-[140px] sm:w-[160px] md:w-[180px] flex-shrink-0"
              style={{ scrollSnapAlign: 'start' } as React.CSSProperties}
            />
          ))
        ) : null}
      </div>
    </div>
  );
};
