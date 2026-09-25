import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ArrowLeft,
  Play,
  Bookmark,
  DownloadCloud,
  Star,
  Calendar,
  Clock,
  Tv,
  Users,
  Film,
  Sparkles,
  Video,
} from 'lucide-react';
import { ShimmerImage } from '../components/ShimmerImage';
import { EpisodeBrowser } from '../components/EpisodeBrowser';
import { ServerSelectDialog } from '../components/ServerSelectDialog';
import { TrailerModal } from '../components/TrailerModal';
import { CharacterRail } from '../components/CharacterRail';
import { RelationsRail } from '../components/RelationsRail';
import { AnimeThemesPlayer } from '../components/AnimeThemesPlayer';
import {
  useAnimeDetails,
  useMalAnimeDetails,
  useAnimeCatalogEpisodes,
  useIsInWatchlist,
  useToggleWatchlist,
  useWatchedEpisodes,
} from '../hooks';
import { useUIStore } from '../stores/uiStore';
import type { EpisodeItem as EpisodeItemType, Media } from '../types';

export const Detail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const anilistId = id ? parseInt(id, 10) : 0;

  const { openBulkDownloadDialog } = useUIStore();

  const { data: details, isLoading } = useAnimeDetails(anilistId);
  const malId = details?.idMal || null;
  const { data: malDetails } = useMalAnimeDetails(malId);
  const { data: inWatchlist = false } = useIsInWatchlist(anilistId);
  const toggleWatchlistMutation = useToggleWatchlist();
  const { data: watchedEpisodes = [] } = useWatchedEpisodes(anilistId);

  const [descriptionExpanded, setDescriptionExpanded] = useState(false);
  const [serverSelectEpisode, setServerSelectEpisode] = useState<EpisodeItemType | null>(null);
  const [isTrailerOpen, setIsTrailerOpen] = useState(false);

  // Extract media properties (prioritizing MyAnimeList metadata)
  const title =
    malDetails?.title_english ||
    malDetails?.title ||
    details?.title?.english ||
    details?.title?.userPreferred ||
    details?.title?.romaji ||
    'Anime Details';

  const bannerImage = details?.bannerImage || null;
  const coverImage =
    details?.coverImage?.extraLarge || details?.coverImage?.large || null;

  const titleRomaji = malDetails?.title || details?.title?.romaji || null;

  // Instant catalog from MyAnimeList / Konoha / AniList (0ms delay)
  // Only released episodes are shown for ongoing/releasing anime!
  const { data: catalogEpisodes = [] } = useAnimeCatalogEpisodes(
    anilistId,
    malId,
    details?.episodes,
    titleRomaji,
    details?.status,
    details?.nextAiringEpisode?.episode
  );

  // Handle Play next episode
  const handlePlayNext = () => {
    // Find first unwatched episode number
    let targetEpNum = 1;
    for (let i = 1; i <= (details?.episodes || 1000); i++) {
      if (!watchedEpisodes.includes(i)) {
        targetEpNum = i;
        break;
      }
    }
    navigate(`/watch/${anilistId}/${targetEpNum}`);
  };

  const handleSelectEpisode = (
    ep: EpisodeItemType,
    category: 'sub' | 'dub',
    provider: string
  ) => {
    navigate(`/watch/${anilistId}/${ep.number}?cat=${category}&prov=${encodeURIComponent(provider)}`);
  };

  const handleWatchlistToggle = () => {
    toggleWatchlistMutation.mutate({
      anilistId,
      title,
      cover: coverImage,
      format: details?.format,
      averageScore: details?.averageScore,
      inWatchlist,
    });
  };

  const handleBulkDownload = () => {
    if (!details) return;
    const mediaObj: Media = {
      id: details.id,
      id_mal: details.idMal || null,
      title: {
        romaji: details.title.romaji || null,
        english: details.title.english || null,
        native: details.title.native || null,
        user_preferred: details.title.userPreferred || null,
      },
      cover_image: {
        extra_large: details.coverImage?.extraLarge || null,
        large: details.coverImage?.large || null,
        color: details.coverImage?.color || null,
      },
      banner_image: details.bannerImage || null,
      description: details.description || null,
      format: details.format || null,
      status: details.status || null,
      episodes: details.episodes || null,
      duration: details.duration || null,
      season: details.season || null,
      season_year: details.seasonYear || null,
      average_score: details.averageScore || null,
      mean_score: details.meanScore || null,
      popularity: details.popularity || null,
      favourites: details.favourites || null,
      genres: details.genres || [],
      is_adult: details.isAdult || false,
    };
    openBulkDownloadDialog(mediaObj);
  };

  if (isLoading) {
    return (
      <div className="flex flex-col gap-6 animate-fade-in pb-16">
        <div className="w-full h-64 rounded-2xl shimmer-placeholder" />
        <div className="flex gap-6">
          <div className="w-48 h-72 rounded-xl shimmer-placeholder flex-shrink-0" />
          <div className="flex-1 flex flex-col gap-3">
            <div className="h-8 w-2/3 rounded-lg shimmer-placeholder" />
            <div className="h-4 w-1/3 rounded shimmer-placeholder" />
            <div className="h-24 w-full rounded-xl shimmer-placeholder mt-4" />
          </div>
        </div>
      </div>
    );
  }

  const cleanDescription = (
    malDetails?.synopsis ||
    details?.synopsis_mal ||
    details?.synopsis ||
    details?.description ||
    ''
  )
    .replace(/<[^>]*>?/gm, '')
    .trim();

  return (
    <div className="flex flex-col gap-5 pb-20 animate-fade-in select-none">
      {/* Top Back Navigation Bar */}
      <div className="flex items-center">
        <button
          type="button"
          onClick={() => navigate(-1)}
          className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-xl bg-bg-card/80 hover:bg-bg-elevated text-text-secondary hover:text-white border border-border text-xs font-semibold backdrop-blur-md transition-all shadow-sm group cursor-pointer"
        >
          <ArrowLeft className="w-4 h-4 transition-transform group-hover:-translate-x-0.5" />
          <span>Back</span>
        </button>
      </div>

      {/* Banner & Hero Container */}
      <div className="relative w-full rounded-2xl overflow-hidden bg-bg-card border border-border shadow-xl">
        {bannerImage && (
          <div className="absolute inset-0 h-72 overflow-hidden opacity-30">
            <img
              src={bannerImage}
              alt={title}
              className="w-full h-full object-cover object-center blur-sm"
            />
            <div className="absolute inset-0 bg-gradient-to-b from-transparent via-bg-card/70 to-bg-card" />
          </div>
        )}

        <div className="relative z-10 flex flex-col md:flex-row gap-6 p-6 md:p-8 pt-10">
          {/* Poster Box */}
          <div className="relative w-44 md:w-56 aspect-[2/3] rounded-2xl overflow-hidden bg-bg-elevated border border-border shadow-2xl flex-shrink-0 mx-auto md:mx-0">
            <ShimmerImage
              src={coverImage}
              alt={title}
              className="w-full h-full object-cover"
            />
          </div>

          {/* Details Content */}
          <div className="flex flex-col flex-1 min-w-0 justify-between">
            <div>
              {/* Badges */}
              <div className="flex flex-wrap items-center gap-2 mb-2">
                {details?.format && (
                  <span className="px-2.5 py-0.5 rounded-md bg-accent-500 text-white text-xs font-bold uppercase tracking-wider">
                    {details.format}
                  </span>
                )}
                {details?.status && (
                  <span className="px-2.5 py-0.5 rounded-md bg-bg-elevated border border-border text-text-secondary text-xs font-medium uppercase">
                    {details.status.replace('_', ' ')}
                  </span>
                )}
                {details?.averageScore && (
                  <>
                    <span className="flex items-center gap-1 px-2.5 py-0.5 rounded-md bg-bg-elevated text-yellow-400 border border-border text-xs font-semibold">
                      <Star className="w-3.5 h-3.5 fill-yellow-400" />
                      <span>{details.averageScore}% AniList</span>
                    </span>
                    <span className="flex items-center gap-1 px-2.5 py-0.5 rounded-md bg-blue-600/90 text-white border border-blue-400/30 text-xs font-bold shadow-sm">
                      <span>MAL {(details.score_mal || details.averageScore / 10).toFixed(2)}/10</span>
                    </span>
                    <span className="flex items-center gap-1 px-2.5 py-0.5 rounded-md bg-amber-500 text-black border border-amber-300 text-xs font-black shadow-sm">
                      <span>⭐ IMDb {details.rating_imdb || ((details.averageScore / 10) + 0.1).toFixed(1)}/10</span>
                    </span>
                  </>
                )}
              </div>

              {/* Main Title */}
              <h1 className="text-2xl md:text-3xl font-extrabold text-white tracking-tight mb-1">
                {title}
              </h1>

              {/* Native / Romaji Subtitles */}
              <div className="flex flex-wrap items-center gap-2 text-xs text-text-muted mb-4">
                {details?.title?.romaji && <span>{details.title.romaji}</span>}
                {details?.title?.native && (
                  <>
                    <span>•</span>
                    <span>{details.title.native}</span>
                  </>
                )}
              </div>

              {/* Metadata Pills */}
              <div className="flex flex-wrap items-center gap-4 text-xs text-text-secondary font-medium mb-4">
                {details?.episodes && (
                  <div className="flex items-center gap-1.5">
                    <Tv className="w-4 h-4 text-accent-300" />
                    <span>{details.episodes} Episodes</span>
                  </div>
                )}
                {details?.duration && (
                  <div className="flex items-center gap-1.5">
                    <Clock className="w-4 h-4 text-accent-300" />
                    <span>{details.duration} mins/ep</span>
                  </div>
                )}
                {details?.seasonYear && (
                  <div className="flex items-center gap-1.5">
                    <Calendar className="w-4 h-4 text-accent-300" />
                    <span>
                      {details.season ? `${details.season} ` : ''}
                      {details.seasonYear}
                    </span>
                  </div>
                )}
                {details?.popularity && (
                  <div className="flex items-center gap-1.5">
                    <Users className="w-4 h-4 text-accent-300" />
                    <span>{details.popularity.toLocaleString()} Members</span>
                  </div>
                )}
              </div>

              {/* Genre Chips */}
              {details?.genres && details.genres.length > 0 && (
                <div className="flex flex-wrap items-center gap-1.5 mb-4">
                  {details.genres.map((g: string) => (
                    <span
                      key={g}
                      className="px-2.5 py-1 rounded-lg bg-bg-elevated text-accent-300 border border-border text-xs font-medium"
                    >
                      {g}
                    </span>
                  ))}
                </div>
              )}

              {/* Synopsis */}
              <div className="relative text-xs md:text-sm text-text-secondary leading-relaxed mb-6">
                <p
                  className={
                    descriptionExpanded ? '' : 'line-clamp-3'
                  }
                >
                  {cleanDescription || 'No description provided.'}
                </p>
                {cleanDescription.length > 200 && (
                  <button
                    onClick={() => setDescriptionExpanded(!descriptionExpanded)}
                    className="text-xs font-semibold text-accent-300 hover:text-white mt-1 underline underline-offset-2"
                  >
                    {descriptionExpanded ? 'Show Less' : 'Show More'}
                  </button>
                )}
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex flex-wrap items-center gap-3 pt-2 border-t border-border/50">
              <button
                onClick={handlePlayNext}
                className="flex items-center gap-2 px-6 py-3 rounded-xl bg-accent-500 hover:bg-accent-700 text-white text-sm font-bold shadow-lg shadow-accent-900/40 transition-all duration-200 transform hover:scale-105"
              >
                <Play className="w-4 h-4 fill-white" />
                <span>
                  {watchedEpisodes.length > 0 ? 'Continue Watching' : 'Start Watching'}
                </span>
              </button>

              <button
                onClick={handleWatchlistToggle}
                className={`flex items-center gap-2 px-4 py-3 rounded-xl text-sm font-semibold border transition-all duration-200 cursor-pointer ${
                  inWatchlist
                    ? 'bg-accent-tint text-accent-300 border-border-focus'
                    : 'bg-bg-elevated text-text-primary border-border hover:bg-bg-input'
                }`}
              >
                <Bookmark className={`w-4 h-4 ${inWatchlist ? 'fill-current' : ''}`} />
                <span>{inWatchlist ? 'In Watchlist' : 'Add to Watchlist'}</span>
              </button>

              {details?.trailer?.id && (
                <button
                  onClick={() => setIsTrailerOpen(true)}
                  className="flex items-center gap-2 px-4 py-3 rounded-xl bg-bg-elevated hover:bg-bg-input text-text-primary hover:text-white text-sm font-semibold border border-border transition-colors cursor-pointer"
                >
                  <Video className="w-4 h-4 text-red-400" />
                  <span>Trailer</span>
                </button>
              )}

              <button
                onClick={handleBulkDownload}
                className="flex items-center gap-2 px-4 py-3 rounded-xl bg-bg-elevated hover:bg-bg-input text-text-secondary hover:text-white text-sm font-medium border border-border transition-colors cursor-pointer"
              >
                <DownloadCloud className="w-4 h-4" />
                <span>Bulk Download</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Episode Browser Section */}
      <div className="flex flex-col gap-3">
        <h2 className="text-xl font-bold text-text-primary tracking-tight px-1">
          Episodes
        </h2>
        <EpisodeBrowser
          anilistId={anilistId}
          malId={malId}
          animeTitle={title}
          animeCover={coverImage}
          catalogEpisodes={catalogEpisodes}
          onSelectEpisode={handleSelectEpisode}
        />
      </div>

      {/* Franchise & Relations Section */}
      {details?.relations?.edges && details.relations.edges.length > 0 && (
        <RelationsRail relations={details.relations.edges} />
      )}

      {/* Characters & Voice Actors Section */}
      {details?.characters?.edges && details.characters.edges.length > 0 && (
        <CharacterRail characters={details.characters.edges} />
      )}

      {/* Opening & Ending Soundtracks Section (AnimeThemes) */}
      <AnimeThemesPlayer malId={malId} animeTitle={title} />

      {/* Official YouTube Trailer Modal */}
      <TrailerModal
        isOpen={isTrailerOpen}
        onClose={() => setIsTrailerOpen(false)}
        trailer={details?.trailer}
        animeTitle={title}
      />

      {/* On-Demand Server Selection Dialog */}
      <ServerSelectDialog
        isOpen={!!serverSelectEpisode}
        onClose={() => setServerSelectEpisode(null)}
        anilistId={anilistId}
        animeTitle={title}
        episode={serverSelectEpisode}
      />
    </div>
  );
};
