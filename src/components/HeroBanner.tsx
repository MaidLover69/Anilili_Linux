import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Play, Info, Star, ChevronLeft, ChevronRight } from 'lucide-react';
import type { Media } from '../types';

interface HeroBannerProps {
  items: Media[];
}

export const HeroBanner: React.FC<HeroBannerProps> = ({ items }) => {
  const navigate = useNavigate();
  const [currentIndex, setCurrentIndex] = useState(0);
  const intervalRef = useRef<number | null>(null);

  const nextSlide = useCallback(() => {
    if (items.length <= 1) return;
    setCurrentIndex((prev) => (prev + 1) % items.length);
  }, [items.length]);

  const resetInterval = useCallback(() => {
    if (intervalRef.current) window.clearInterval(intervalRef.current);
    if (items.length > 1) {
      intervalRef.current = window.setInterval(nextSlide, 7000);
    }
  }, [items.length, nextSlide]);

  useEffect(() => {
    resetInterval();
    return () => {
      if (intervalRef.current) window.clearInterval(intervalRef.current);
    };
  }, [resetInterval]);

  if (!items || items.length === 0) return null;

  const current = items[currentIndex];
  if (!current) return null;

  const title =
    current.title.english ||
    current.title.user_preferred ||
    current.title.romaji ||
    'Untitled';

  const backdropUrl =
    current.banner_image ||
    current.cover_image.extra_large ||
    current.cover_image.large ||
    '';

  const cleanDescription = (current.description || '')
    .replace(/<[^>]*>?/gm, '')
    .trim();

  const handleWatch = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    navigate(`/watch/${current.id}/1`);
  };

  const handleDetails = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    navigate(`/detail/${current.id}`);
  };

  return (
    <div className="relative w-full h-[380px] md:h-[440px] rounded-2xl overflow-hidden bg-bg-card border border-border shadow-2xl group select-none">
      {/* Background Image with Parallax / Zoom Effect */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <img
          key={current.id}
          src={backdropUrl}
          alt={title}
          className="w-full h-full object-cover object-center transform scale-105 transition-all duration-1000 ease-out brightness-75"
        />
        {/* Layered Gradient Overlays */}
        <div className="absolute inset-0 bg-gradient-to-t from-bg-page via-bg-page/60 to-transparent pointer-events-none" />
        <div className="absolute inset-0 bg-gradient-to-r from-bg-page via-bg-page/80 to-transparent w-full md:w-3/4 pointer-events-none" />
      </div>

      {/* Content Container */}
      <div className="relative z-20 flex flex-col justify-end h-full p-6 md:p-10 max-w-2xl pointer-events-auto">
        {/* Badges */}
        <div className="flex items-center gap-2 mb-3">
          {current.format && (
            <span className="px-2.5 py-0.5 text-xs font-bold uppercase tracking-wider rounded-md bg-accent-500 text-white shadow-sm">
              {current.format}
            </span>
          )}
          {current.rating_imdb && (
            <span className="flex items-center gap-1 px-2.5 py-0.5 text-xs font-black rounded-md bg-amber-500 text-black shadow-sm">
              IMDb {current.rating_imdb}
            </span>
          )}
          {current.score_mal && (
            <span className="flex items-center gap-1 px-2.5 py-0.5 text-xs font-bold rounded-md bg-blue-600 text-white shadow-sm">
              MAL {current.score_mal.toFixed(2)}
            </span>
          )}
          {current.average_score && (
            <span className="flex items-center gap-1 px-2.5 py-0.5 text-xs font-semibold rounded-md bg-bg-elevated/80 text-yellow-400 border border-border backdrop-blur-md">
              <Star className="w-3.5 h-3.5 fill-yellow-400" />
              <span>{current.average_score}%</span>
            </span>
          )}
          {current.season_year && (
            <span className="text-xs text-text-muted font-medium">
              {current.season ? `${current.season} ` : ''}{current.season_year}
            </span>
          )}
        </div>

        {/* Anime Title */}
        <h1
          onClick={handleDetails}
          className="text-2xl md:text-4xl font-extrabold text-white tracking-tight line-clamp-2 drop-shadow-md mb-2 cursor-pointer hover:text-accent-300 transition-colors"
        >
          {title}
        </h1>

        {/* Genres */}
        {current.genres && current.genres.length > 0 && (
          <div className="flex items-center gap-2 mb-3 text-xs text-accent-300 font-medium">
            {current.genres.slice(0, 4).map((g) => (
              <span key={g} className="px-2 py-0.5 rounded bg-accent-tint border border-border-focus">
                {g}
              </span>
            ))}
          </div>
        )}

        {/* Synopsis */}
        <p className="text-xs md:text-sm text-text-secondary line-clamp-3 mb-5 leading-relaxed">
          {cleanDescription || 'No description available for this anime title.'}
        </p>

        {/* Action Buttons */}
        <div className="flex items-center gap-3 relative z-30 pointer-events-auto">
          <button
            type="button"
            onClick={handleWatch}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-accent-500 hover:bg-accent-700 active:scale-95 text-white text-sm font-semibold shadow-lg shadow-accent-900/40 transition-all duration-200 transform hover:scale-105 cursor-pointer"
          >
            <Play className="w-4 h-4 fill-white" />
            <span>Watch Episode 1</span>
          </button>
          <button
            type="button"
            onClick={handleDetails}
            className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-bg-elevated/90 hover:bg-bg-input active:scale-95 text-text-primary text-sm font-medium border border-border backdrop-blur-md transition-all duration-200 hover:border-border-focus cursor-pointer"
          >
            <Info className="w-4 h-4" />
            <span>Details</span>
          </button>
        </div>
      </div>

      {/* Slide Navigation Controls */}
      {items.length > 1 && (
        <>
          <button
            type="button"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              setCurrentIndex((prev) => (prev - 1 + items.length) % items.length);
              resetInterval();
            }}
            aria-label="Previous banner"
            className="absolute left-3 top-1/2 -translate-y-1/2 p-2.5 rounded-full bg-bg-deepest/80 hover:bg-bg-deepest text-white backdrop-blur-md border border-border opacity-0 group-hover:opacity-100 transition-all duration-200 z-30 cursor-pointer pointer-events-auto shadow-lg hover:scale-110"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <button
            type="button"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              setCurrentIndex((prev) => (prev + 1) % items.length);
              resetInterval();
            }}
            aria-label="Next banner"
            className="absolute right-3 top-1/2 -translate-y-1/2 p-2.5 rounded-full bg-bg-deepest/80 hover:bg-bg-deepest text-white backdrop-blur-md border border-border opacity-0 group-hover:opacity-100 transition-all duration-200 z-30 cursor-pointer pointer-events-auto shadow-lg hover:scale-110"
          >
            <ChevronRight className="w-5 h-5" />
          </button>

          {/* Dots Indicator */}
          <div className="absolute bottom-4 right-6 flex items-center gap-1.5 z-30 pointer-events-auto">
            {items.map((_, idx) => (
              <button
                key={idx}
                type="button"
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  setCurrentIndex(idx);
                }}
                aria-label={`Go to slide ${idx + 1}`}
                className={`h-2 rounded-full transition-all duration-300 cursor-pointer ${
                  currentIndex === idx
                    ? 'w-6 bg-accent-500 shadow-md shadow-accent-500/50'
                    : 'w-2 bg-white/30 hover:bg-white/60'
                }`}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
};

export default HeroBanner;
