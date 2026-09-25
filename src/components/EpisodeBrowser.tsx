import React, { useState, useMemo } from 'react';
import {
  Search,
  Volume2,
  Mic,
  Layers,
  Filter,
  List,
  Grid,
  Menu,
  ListPlus,
} from 'lucide-react';
import { EpisodeItem } from './EpisodeItem';
import { CustomSelect } from './CustomSelect';
import { AddToPlaylistModal } from './AddToPlaylistModal';
import type { EpisodeItem as EpisodeItemType, EpisodesResult } from '../types';
import { useWatchedEpisodes, useFillerEpisodes } from '../hooks';

interface EpisodeBrowserProps {
  anilistId: number;
  malId?: number | null;
  animeTitle: string;
  animeCover: string | null;
  catalogEpisodes?: EpisodeItemType[];
  episodesData?: EpisodesResult;
  currentEpisodeNumber?: number | null;
  onSelectEpisode: (ep: EpisodeItemType, category: 'sub' | 'dub', provider: string) => void;
  onDownloadEpisode?: (ep: EpisodeItemType, category: 'sub' | 'dub', provider: string) => void;
}

const CHUNK_SIZE = 100;

export const EpisodeBrowser: React.FC<EpisodeBrowserProps> = ({
  anilistId,
  malId,
  animeTitle,
  animeCover,
  catalogEpisodes = [],
  episodesData = {},
  currentEpisodeNumber,
  onSelectEpisode,
  onDownloadEpisode,
}) => {
  const [category, setCategory] = useState<'sub' | 'dub'>('sub');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedChunkIndex, setSelectedChunkIndex] = useState(0);
  const [viewMode, setViewMode] = useState<'compact' | 'list' | 'grid'>('list');
  const [playlistModalOpen, setPlaylistModalOpen] = useState(false);
  const [playlistTargetEpisodes, setPlaylistTargetEpisodes] = useState<EpisodeItemType[]>([]);

  // Filter to ONLY providers that returned sub.length > 0 or dub.length > 0
  const providerNames = useMemo(() => {
    return Object.keys(episodesData).filter((p) => {
      const data = episodesData[p];
      return (data?.sub && data.sub.length > 0) || (data?.dub && data.dub.length > 0);
    });
  }, [episodesData]);

  const [selectedProvider, setSelectedProvider] = useState<string>('');

  // Fallback to active provider if list updates
  const activeProvider =
    selectedProvider && providerNames.includes(selectedProvider)
      ? selectedProvider
      : providerNames[0] || 'auto';

  const providerData = episodesData[activeProvider] || { sub: [], dub: [] };
  const providerEpisodes = category === 'dub' ? providerData.dub : providerData.sub;
  const allEpisodes = providerEpisodes.length > 0 ? providerEpisodes : catalogEpisodes;

  const { data: watchedEpisodes = [] } = useWatchedEpisodes(anilistId);
  const { data: fillerEpisodes = [] } = useFillerEpisodes(malId);

  // Chunk calculations
  const totalEpisodes = allEpisodes.length;
  const chunkCount = Math.max(1, Math.ceil(totalEpisodes / CHUNK_SIZE));

  const chunks = useMemo(() => {
    const list = [];
    for (let i = 0; i < chunkCount; i++) {
      const start = i * CHUNK_SIZE + 1;
      const end = Math.min((i + 1) * CHUNK_SIZE, totalEpisodes || 100);
      list.push({ index: i, label: `${start}-${end}`, start, end });
    }
    return list;
  }, [chunkCount, totalEpisodes]);

  // Virtualized Slice: Max 100 items rendered concurrently
  const visibleEpisodes = useMemo(() => {
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      return allEpisodes
        .filter(
          (ep) =>
            ep.number.toString().includes(q) ||
            (ep.title && ep.title.toLowerCase().includes(q))
        )
        .slice(0, CHUNK_SIZE);
    }

    const startIdx = selectedChunkIndex * CHUNK_SIZE;
    const endIdx = startIdx + CHUNK_SIZE;
    return allEpisodes.slice(startIdx, endIdx);
  }, [allEpisodes, searchQuery, selectedChunkIndex]);

  // Has DUB available
  const hasDub = providerData.dub && providerData.dub.length > 0;

  return (
    <div className="flex flex-col gap-4 w-full select-none">
      {/* Top Controls Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-xl bg-bg-card border border-border">
        {/* Left: Sub / Dub Toggle + View Mode Toggle */}
        <div className="flex flex-wrap items-center gap-2">
          {/* Sub / Dub Toggle */}
          <div className="flex items-center p-1 rounded-lg bg-bg-elevated border border-border">
            <button
              onClick={() => setCategory('sub')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-bold transition-all duration-200 ${
                category === 'sub'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <Volume2 className="w-3.5 h-3.5" />
              <span>SUB ({providerData.sub?.length || 0})</span>
            </button>
            <button
              onClick={() => hasDub && setCategory('dub')}
              disabled={!hasDub}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-bold transition-all duration-200 ${
                !hasDub
                  ? 'opacity-40 cursor-not-allowed text-text-disabled'
                  : category === 'dub'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <Mic className="w-3.5 h-3.5" />
              <span>DUB ({providerData.dub?.length || 0})</span>
            </button>
          </div>

          {/* View Mode Toggle Pill (Compact / List / Grid) */}
          <div className="flex items-center p-1 rounded-lg bg-bg-elevated border border-border">
            <button
              type="button"
              onClick={() => setViewMode('compact')}
              title="Compact Mode"
              className={`p-1.5 rounded-md transition-all ${
                viewMode === 'compact'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <Menu className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={() => setViewMode('list')}
              title="List Mode"
              className={`p-1.5 rounded-md transition-all ${
                viewMode === 'list'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <List className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={() => setViewMode('grid')}
              title="Grid Mode"
              className={`p-1.5 rounded-md transition-all ${
                viewMode === 'grid'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <Grid className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Right: Provider Selector & Instant Search Filter */}
        <div className="flex flex-wrap items-center gap-3">
          {/* Provider Selector */}
          {providerNames.length > 1 && (
            <div className="flex items-center gap-2">
              <span className="text-xs text-text-muted font-medium">Provider:</span>
              <CustomSelect
                value={activeProvider}
                onChange={(val) => {
                  setSelectedProvider(val);
                  setSelectedChunkIndex(0);
                }}
                options={providerNames}
              />
            </div>
          )}

          {/* Add to Playlist Button */}
          <button
            type="button"
            onClick={() => {
              setPlaylistTargetEpisodes(visibleEpisodes);
              setPlaylistModalOpen(true);
            }}
            title="Add visible episodes to a custom playlist"
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-bg-elevated hover:bg-accent-500/10 hover:text-accent-300 border border-border hover:border-accent-500/30 text-xs font-semibold text-text-secondary transition-all cursor-pointer"
          >
            <ListPlus className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Add to Playlist</span>
          </button>

          {/* Instant Episode Search Filter */}
          <div className="relative min-w-[200px]">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-muted" />
            <input
              type="text"
              placeholder="🔍 Search episodes..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-8 pr-3 py-1.5 rounded-lg bg-bg-input border border-border text-xs text-text-primary placeholder:text-text-muted focus:border-border-focus outline-none transition-colors"
            />
          </div>
        </div>
      </div>

      {/* Chunk Tabs (100-Episode Range Selector) */}
      {!searchQuery && chunks.length > 1 && (
        <div className="flex items-center gap-2 overflow-x-auto pb-1 no-scrollbar">
          <span className="text-xs text-text-muted font-medium flex items-center gap-1 flex-shrink-0">
            <Layers className="w-3.5 h-3.5" /> Range:
          </span>
          {chunks.map((chunk) => (
            <button
              key={chunk.index}
              onClick={() => setSelectedChunkIndex(chunk.index)}
              className={`px-3 py-1 text-xs font-semibold rounded-lg flex-shrink-0 transition-all duration-200 ${
                selectedChunkIndex === chunk.index
                  ? 'bg-accent-tint text-accent-300 border border-border-focus font-bold'
                  : 'bg-bg-elevated text-text-secondary hover:text-white border border-border'
              }`}
            >
              {chunk.label}
            </button>
          ))}
        </div>
      )}

      {/* Episodes Container (Guaranteed max 100 rendered) */}
      {visibleEpisodes.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-12 bg-bg-card rounded-xl border border-border text-center">
          <Filter className="w-8 h-8 text-text-muted mb-2 opacity-50" />
          <p className="text-sm font-semibold text-text-secondary">
            No episodes found
          </p>
          <span className="text-xs text-text-muted mt-1">
            Try switching providers or changing your search criteria.
          </span>
        </div>
      ) : (
        <div
          className={`max-h-[560px] overflow-y-auto pr-1 ${
            viewMode === 'compact'
              ? 'flex flex-col gap-1.5'
              : viewMode === 'grid'
              ? 'grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3'
              : 'grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-3 gap-2.5'
          }`}
        >
          {visibleEpisodes.map((ep) => {
            const isWatched = watchedEpisodes.some(
              (we) => Math.abs(we - ep.number) < 0.01
            );
            const isFiller = fillerEpisodes.includes(Math.round(ep.number));
            const isActive =
              currentEpisodeNumber !== undefined &&
              currentEpisodeNumber !== null &&
              Math.abs(currentEpisodeNumber - ep.number) < 0.01;

            return (
              <EpisodeItem
                key={`${ep.pipe_id}-${ep.number}`}
                episode={ep}
                animeId={anilistId}
                animeTitle={animeTitle}
                animeCover={animeCover}
                isActive={isActive}
                isWatched={isWatched}
                isFiller={isFiller}
                viewMode={viewMode}
                onSelect={(selectedEp) =>
                  onSelectEpisode(selectedEp, category, activeProvider)
                }
                onDownload={
                  onDownloadEpisode
                    ? (downloadEp) =>
                        onDownloadEpisode(downloadEp, category, activeProvider)
                    : undefined
                }
                onAddToPlaylist={(targetEp) => {
                  setPlaylistTargetEpisodes([targetEp]);
                  setPlaylistModalOpen(true);
                }}
              />
            );
          })}
        </div>
      )}

      {/* Add to Playlist Modal */}
      {playlistModalOpen && (
        <AddToPlaylistModal
          isOpen={playlistModalOpen}
          onClose={() => setPlaylistModalOpen(false)}
          anilistId={anilistId}
          malId={malId}
          seriesTitle={animeTitle}
          seriesCover={animeCover}
          category={category}
          episodes={playlistTargetEpisodes}
        />
      )}
    </div>
  );
};
