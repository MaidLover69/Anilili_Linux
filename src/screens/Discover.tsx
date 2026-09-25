import React, { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Search, Filter, ChevronLeft, ChevronRight, X, Sparkles } from 'lucide-react';
import { AnimeCard } from '../components/AnimeCard';
import { SkeletonCard } from '../components/SkeletonCard';
import { CustomSelect } from '../components/CustomSelect';
import { useSearchAnime } from '../hooks/useAniList';

const GENRES = [
  'Action',
  'Adventure',
  'Comedy',
  'Drama',
  'Fantasy',
  'Horror',
  'Mahou Shoujo',
  'Mecha',
  'Music',
  'Mystery',
  'Psychological',
  'Romance',
  'Sci-Fi',
  'Slice of Life',
  'Sports',
  'Supernatural',
  'Thriller',
];

const FORMATS = [
  { label: 'All Formats', value: '' },
  { label: 'TV Show', value: 'TV' },
  { label: 'Movie', value: 'MOVIE' },
  { label: 'TV Short', value: 'TV_SHORT' },
  { label: 'OVA', value: 'OVA' },
  { label: 'ONA', value: 'ONA' },
  { label: 'Special', value: 'SPECIAL' },
];

const STATUSES = [
  { label: 'All Statuses', value: '' },
  { label: 'Releasing', value: 'RELEASING' },
  { label: 'Finished', value: 'FINISHED' },
  { label: 'Not Yet Released', value: 'NOT_YET_RELEASED' },
  { label: 'Cancelled', value: 'CANCELLED' },
];

const SORTS = [
  { label: 'Popularity', value: 'POPULARITY_DESC' },
  { label: 'Trending', value: 'TRENDING_DESC' },
  { label: 'Score', value: 'SCORE_DESC' },
  { label: 'Newest', value: 'START_DATE_DESC' },
  { label: 'Favorites', value: 'FAVOURITES_DESC' },
  { label: 'Title', value: 'TITLE_ROMAJI' },
];

export const Discover: React.FC = () => {
  const [searchParams, setSearchParams] = useSearchParams();

  const [query, setQuery] = useState(searchParams.get('q') || '');
  const [selectedGenres, setSelectedGenres] = useState<string[]>(
    searchParams.getAll('genre') || []
  );
  const [format, setFormat] = useState(searchParams.get('format') || '');
  const [status, setStatus] = useState(searchParams.get('status') || '');
  const [sort, setSort] = useState(searchParams.get('sort') || 'POPULARITY_DESC');
  const [page, setPage] = useState(1);

  // Sync sort from searchParams if navigated with link (e.g. from View All)
  useEffect(() => {
    const s = searchParams.get('sort');
    if (s && s !== sort) setSort(s);
  }, [searchParams]);

  const { data: animeList = [], isLoading, isFetching } = useSearchAnime({
    query: query.trim() || undefined,
    genres: selectedGenres.length > 0 ? selectedGenres : undefined,
    format: format || undefined,
    status: status || undefined,
    sort,
    page,
    perPage: 24,
  });

  const toggleGenre = (genre: string) => {
    setPage(1);
    setSelectedGenres((prev) =>
      prev.includes(genre) ? prev.filter((g) => g !== genre) : [...prev, genre]
    );
  };

  const handleClearFilters = () => {
    setQuery('');
    setSelectedGenres([]);
    setFormat('');
    setStatus('');
    setSort('POPULARITY_DESC');
    setPage(1);
  };

  const hasActiveFilters =
    query || selectedGenres.length > 0 || format || status || sort !== 'POPULARITY_DESC';

  return (
    <div className="flex flex-col gap-6 pb-16 animate-fade-in select-none">
      {/* Header Search & Filter Bar */}
      <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card border border-border shadow-lg">
        <div className="flex flex-col md:flex-row items-stretch md:items-center gap-3">
          {/* Main Search Input */}
          <div className="relative flex-1">
            <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" />
            <input
              type="text"
              placeholder="Search anime title, studio, characters..."
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setPage(1);
              }}
              className="w-full pl-10 pr-4 py-2.5 rounded-xl bg-bg-input border border-border text-sm text-text-primary placeholder:text-text-muted focus:border-border-focus outline-none transition-colors"
            />
            {query && (
              <button
                onClick={() => setQuery('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 p-1 rounded-md text-text-muted hover:text-white"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>

          {/* Format Selector */}
          <CustomSelect
            value={format}
            onChange={(val) => {
              setFormat(val);
              setPage(1);
            }}
            options={FORMATS}
          />

          {/* Status Selector */}
          <CustomSelect
            value={status}
            onChange={(val) => {
              setStatus(val);
              setPage(1);
            }}
            options={STATUSES}
          />

          {/* Sort Selector */}
          <CustomSelect
            value={sort}
            onChange={(val) => {
              setSort(val);
              setPage(1);
            }}
            options={SORTS.map((s) => ({ value: s.value, label: `Sort: ${s.label}` }))}
          />

          {/* Reset Filters */}
          {hasActiveFilters && (
            <button
              onClick={handleClearFilters}
              className="flex items-center justify-center gap-1.5 px-3.5 py-2 rounded-xl bg-bg-elevated hover:bg-state-error/20 text-text-muted hover:text-state-error border border-border text-xs font-medium transition-colors"
            >
              <X className="w-3.5 h-3.5" />
              <span>Reset</span>
            </button>
          )}
        </div>

        {/* Genre Pill Chips */}
        <div className="flex items-center gap-1.5 overflow-x-auto pb-1 no-scrollbar">
          {GENRES.map((genre) => {
            const isSelected = selectedGenres.includes(genre);
            return (
              <button
                key={genre}
                onClick={() => toggleGenre(genre)}
                className={`px-3 py-1 rounded-lg text-xs font-medium whitespace-nowrap transition-all duration-200 ${
                  isSelected
                    ? 'bg-accent-500 text-white font-semibold shadow-md shadow-accent-900/30'
                    : 'bg-bg-elevated text-text-secondary hover:text-white border border-border'
                }`}
              >
                {genre}
              </button>
            );
          })}
        </div>
      </div>

      {/* Grid Results */}
      {isLoading ? (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
          {Array.from({ length: 24 }).map((_, i) => (
            <SkeletonCard key={i} />
          ))}
        </div>
      ) : animeList.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 bg-bg-card rounded-2xl border border-border text-center">
          <Sparkles className="w-10 h-10 text-text-muted mb-3 opacity-40" />
          <h3 className="text-base font-bold text-text-primary">No Anime Found</h3>
          <p className="text-xs text-text-muted mt-1 max-w-sm">
            Try adjusting your search keywords, clearing genre filters, or choosing a different sort option.
          </p>
          <button
            onClick={handleClearFilters}
            className="mt-4 px-4 py-2 rounded-xl bg-accent-500 hover:bg-accent-700 text-white text-xs font-semibold transition-colors"
          >
            Clear All Filters
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
          {animeList.map((anime) => (
            <AnimeCard key={anime.id} media={anime} />
          ))}
        </div>
      )}

      {/* Pagination Controls */}
      {animeList.length > 0 && (
        <div className="flex items-center justify-center gap-3 pt-6">
          <button
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page <= 1 || isFetching}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-bg-card border border-border text-xs font-semibold text-text-primary hover:border-border-focus disabled:opacity-40 disabled:pointer-events-none transition-all"
          >
            <ChevronLeft className="w-4 h-4" />
            <span>Previous</span>
          </button>

          <span className="px-4 py-2 rounded-xl bg-bg-elevated border border-border text-xs font-bold text-accent-300">
            Page {page}
          </span>

          <button
            onClick={() => setPage((p) => p + 1)}
            disabled={animeList.length < 24 || isFetching}
            className="flex items-center gap-1.5 px-4 py-2 rounded-xl bg-bg-card border border-border text-xs font-semibold text-text-primary hover:border-border-focus disabled:opacity-40 disabled:pointer-events-none transition-all"
          >
            <span>Next</span>
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}
    </div>
  );
};
