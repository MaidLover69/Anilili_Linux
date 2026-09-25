import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Search,
  Film,
  Compass,
  BookOpen,
  Calendar,
  Settings as SettingsIcon,
  X,
  Sparkles,
  ArrowRight,
  TrendingUp,
} from 'lucide-react';
import { useSearchAnime } from '../hooks/useAniList';
import { Media } from '../types';
import { ShimmerImage } from './ShimmerImage';

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
}

interface NavItem {
  id: string;
  title: string;
  subtitle: string;
  path: string;
  icon: React.ReactNode;
}

const STATIC_NAV_ITEMS: NavItem[] = [
  {
    id: 'nav-home',
    title: 'Home',
    subtitle: 'Trending, popular, and continue watching',
    path: '/',
    icon: <Film className="w-4 h-4 text-accent-500" />,
  },
  {
    id: 'nav-discover',
    title: 'Discover Anime',
    subtitle: 'Browse by genre, seasonal filters, and formats',
    path: '/discover',
    icon: <Compass className="w-4 h-4 text-emerald-400" />,
  },
  {
    id: 'nav-library',
    title: 'My Library',
    subtitle: 'Watchlist, history, downloads, and AniList sync',
    path: '/library',
    icon: <BookOpen className="w-4 h-4 text-amber-400" />,
  },
  {
    id: 'nav-schedule',
    title: 'Airing Schedule',
    subtitle: 'Weekly release calendar and countdowns',
    path: '/schedule',
    icon: <Calendar className="w-4 h-4 text-sky-400" />,
  },
  {
    id: 'nav-settings',
    title: 'Settings',
    subtitle: 'Playback, audio priorities, providers & accounts',
    path: '/settings',
    icon: <SettingsIcon className="w-4 h-4 text-purple-400" />,
  },
];

export const CommandPalette: React.FC<CommandPaletteProps> = ({ isOpen, onClose }) => {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Debounced search query
  const [debouncedQuery, setDebouncedQuery] = useState('');
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(query.trim()), 200);
    return () => clearTimeout(timer);
  }, [query]);

  const { data: searchResults, isLoading } = useSearchAnime(
    { query: debouncedQuery.length >= 2 ? debouncedQuery : undefined, perPage: 6 },
    { enabled: debouncedQuery.length >= 2 }
  );

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const displayedAnime: Media[] = searchResults || [];
  const filteredNavItems = STATIC_NAV_ITEMS.filter((item) =>
    item.title.toLowerCase().includes(query.toLowerCase()) ||
    item.subtitle.toLowerCase().includes(query.toLowerCase())
  );

  const totalItems = (displayedAnime.length > 0 ? displayedAnime.length : 0) + filteredNavItems.length;

  useEffect(() => {
    setSelectedIndex(0);
  }, [query, displayedAnime.length]);

  const handleSelect = (idx: number) => {
    if (displayedAnime.length > 0 && idx < displayedAnime.length) {
      const anime = displayedAnime[idx];
      navigate(`/detail/${anime.id}`);
      onClose();
    } else {
      const navIdx = idx - (displayedAnime.length > 0 ? displayedAnime.length : 0);
      const item = filteredNavItems[navIdx];
      if (item) {
        navigate(item.path);
        onClose();
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (totalItems > 0 ? (prev + 1) % totalItems : 0));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (totalItems > 0 ? (prev - 1 + totalItems) % totalItems : 0));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (totalItems > 0) {
        handleSelect(selectedIndex);
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-20 px-4 bg-black/70 backdrop-blur-md animate-fadeIn"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl bg-[#0e0e12] border border-white/10 rounded-2xl shadow-2xl overflow-hidden glass-modal"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* Search Input Bar */}
        <div className="flex items-center px-4 py-3.5 border-b border-white/10 bg-white/[0.02]">
          <Search className="w-5 h-5 text-white/40 mr-3 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            className="w-full bg-transparent text-white placeholder-white/40 text-base outline-none border-none p-0 focus:ring-0"
            placeholder="Search anime, genres, screens (Type to search, ↑↓ to navigate, Enter to select)..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {query ? (
            <button
              onClick={() => setQuery('')}
              className="p-1 rounded-lg hover:bg-white/10 text-white/40 hover:text-white transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          ) : (
            <kbd className="px-2 py-0.5 text-[10px] font-semibold tracking-wider text-white/40 bg-white/5 border border-white/10 rounded-md">
              ESC
            </kbd>
          )}
        </div>

        {/* Results Body */}
        <div className="max-h-[60vh] overflow-y-auto p-2 divide-y divide-white/5">
          {/* Loading Indicator */}
          {isLoading && query.length >= 2 && (
            <div className="flex items-center justify-center py-8 text-white/40 gap-2">
              <div className="w-4 h-4 border-2 border-accent-500 border-t-transparent rounded-full animate-spin" />
              <span className="text-sm">Searching AniList...</span>
            </div>
          )}

          {/* Anime Search Results Section */}
          {displayedAnime.length > 0 && (
            <div className="py-1">
              <div className="px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-white/40 flex items-center gap-1.5">
                <Sparkles className="w-3.5 h-3.5 text-accent-500" />
                Anime Results
              </div>
              <div className="space-y-1">
                {displayedAnime.map((anime, idx) => {
                  const isSelected = selectedIndex === idx;
                  const title = anime.title.english || anime.title.user_preferred || anime.title.romaji || 'Untitled';
                  const cover = anime.cover_image.large || anime.cover_image.extra_large || '';

                  return (
                    <div
                      key={anime.id}
                      onClick={() => handleSelect(idx)}
                      onMouseEnter={() => setSelectedIndex(idx)}
                      className={`flex items-center gap-3 px-3 py-2.5 rounded-xl cursor-pointer transition-all duration-150 ${
                        isSelected
                          ? 'bg-accent-500/20 text-white border border-accent-500/40 shadow-sm'
                          : 'text-white/80 hover:bg-white/5 border border-transparent'
                      }`}
                    >
                      <div className="w-10 h-14 shrink-0 rounded-lg overflow-hidden bg-white/5">
                        <ShimmerImage src={cover} alt={title} className="w-full h-full object-cover" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-sm text-white truncate">{title}</div>
                        <div className="flex items-center gap-2 text-xs text-white/50 mt-0.5">
                          {anime.format && <span className="px-1.5 py-0.5 rounded bg-white/10 text-[10px]">{anime.format}</span>}
                          {anime.season_year && <span>{anime.season_year}</span>}
                          {anime.average_score && (
                            <span className="text-amber-400 font-medium">{anime.average_score}%</span>
                          )}
                          {anime.genres.slice(0, 2).map((g) => (
                            <span key={g} className="text-white/40">
                              • {g}
                            </span>
                          ))}
                        </div>
                      </div>
                      <ArrowRight
                        className={`w-4 h-4 transition-transform ${
                          isSelected ? 'text-accent-500 translate-x-0.5' : 'text-white/20'
                        }`}
                      />
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Quick Navigation Section */}
          {filteredNavItems.length > 0 && (
            <div className="py-1">
              <div className="px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-white/40 flex items-center gap-1.5">
                <Compass className="w-3.5 h-3.5 text-white/40" />
                Navigation & Screens
              </div>
              <div className="space-y-1">
                {filteredNavItems.map((item, navIdx) => {
                  const actualIdx = (displayedAnime.length > 0 ? displayedAnime.length : 0) + navIdx;
                  const isSelected = selectedIndex === actualIdx;

                  return (
                    <div
                      key={item.id}
                      onClick={() => handleSelect(actualIdx)}
                      onMouseEnter={() => setSelectedIndex(actualIdx)}
                      className={`flex items-center gap-3 px-3 py-2.5 rounded-xl cursor-pointer transition-all duration-150 ${
                        isSelected
                          ? 'bg-accent-500/20 text-white border border-accent-500/40 shadow-sm'
                          : 'text-white/80 hover:bg-white/5 border border-transparent'
                      }`}
                    >
                      <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center shrink-0">
                        {item.icon}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-sm text-white">{item.title}</div>
                        <div className="text-xs text-white/50 truncate">{item.subtitle}</div>
                      </div>
                      <kbd
                        className={`px-1.5 py-0.5 text-[10px] font-mono rounded border ${
                          isSelected
                            ? 'bg-accent-500/30 text-white border-accent-500/50'
                            : 'bg-white/5 text-white/30 border-white/10'
                        }`}
                      >
                        Enter ↵
                      </kbd>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Empty State */}
          {!isLoading && query.length >= 2 && displayedAnime.length === 0 && filteredNavItems.length === 0 && (
            <div className="py-12 text-center text-white/40">
              <Film className="w-8 h-8 mx-auto mb-2 opacity-30" />
              <p className="text-sm font-medium">No results found for "{query}"</p>
              <p className="text-xs text-white/30 mt-1">Try searching for a different title or keyword</p>
            </div>
          )}
        </div>

        {/* Footer shortcuts */}
        <div className="flex items-center justify-between px-4 py-2.5 bg-black/40 border-t border-white/5 text-[11px] text-white/40">
          <div className="flex items-center gap-3">
            <span>
              <kbd className="px-1 py-0.5 bg-white/10 rounded mr-1">↑↓</kbd> to navigate
            </span>
            <span>
              <kbd className="px-1 py-0.5 bg-white/10 rounded mr-1">↵</kbd> to select
            </span>
            <span>
              <kbd className="px-1 py-0.5 bg-white/10 rounded mr-1">esc</kbd> to close
            </span>
          </div>
          <span className="text-white/30">Anilili Global Search</span>
        </div>
      </div>
    </div>
  );
};

export default CommandPalette;
