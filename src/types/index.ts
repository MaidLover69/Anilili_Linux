export interface MediaTitle {
  romaji: string | null;
  english: string | null;
  native: string | null;
  user_preferred: string | null;
}

export interface CoverImage {
  extra_large: string | null;
  large: string | null;
  color: string | null;
}

export interface NextAiringEpisode {
  id: number;
  airing_at: number;
  time_until_airing: number;
  episode: number;
}

export interface Trailer {
  id: string | null;
  site: string | null;
  thumbnail: string | null;
}

export interface CharacterName {
  full: string | null;
  native: string | null;
}

export interface CharacterImage {
  large: string | null;
  medium: string | null;
}

export interface Character {
  id: number;
  name: CharacterName | null;
  image: CharacterImage | null;
}

export interface VoiceActor {
  id: number;
  name: CharacterName | null;
  image: CharacterImage | null;
  languageV2?: string | null;
}

export interface CharacterEdge {
  role: string | null;
  node: Character | null;
  voiceActors?: VoiceActor[];
}

export interface RelationEdge {
  relationType: string | null;
  node: {
    id: number;
    idMal?: number | null;
    title: {
      romaji: string | null;
      english: string | null;
      native: string | null;
      userPreferred: string | null;
    };
    coverImage: {
      large: string | null;
      extraLarge: string | null;
      color: string | null;
    };
    format: string | null;
    status: string | null;
    averageScore: number | null;
  };
}

export interface Media {
  id: number;
  id_mal: number | null;
  idMal?: number | null;
  imdb_id?: string | null;
  title: MediaTitle;
  cover_image: CoverImage;
  coverImage?: {
    large?: string | null;
    extraLarge?: string | null;
    medium?: string | null;
    color?: string | null;
  };
  banner_image: string | null;
  bannerImage?: string | null;
  description: string | null;
  synopsis_mal?: string | null;
  synopsis_imdb?: string | null;
  format: string | null;
  status: string | null;
  episodes: number | null;
  duration: number | null;
  season: string | null;
  season_year: number | null;
  seasonYear?: number | null;
  average_score: number | null; // AniList score (e.g. 87)
  averageScore?: number | null;
  score_mal?: number | null;    // MAL rating (e.g. 8.71)
  rating_imdb?: string | null;  // IMDb rating (e.g. "8.9")
  mean_score: number | null;
  meanScore?: number | null;
  popularity: number | null;
  favourites: number | null;
  genres: string[];
  is_adult: boolean;
  isAdult?: boolean;
  next_airing_episode?: NextAiringEpisode | null;
  nextAiringEpisode?: {
    id: number;
    airingAt: number;
    timeUntilAiring: number;
    episode: number;
  } | null;
  trailer?: Trailer | null;
  characters?: {
    edges: CharacterEdge[];
  } | null;
  relations?: {
    edges: RelationEdge[];
  } | null;
}

export interface StreamItem {
  url: string;
  stream_type: string; // 'hls' | 'mp4' | 'dash'
  quality: string | null;
  audio?: string | null;
  subtitle_variant?: 'h-sub' | 's-sub' | null;
  referer?: string | null;
  origin?: string | null;
  headers?: Record<string, string>;
  is_active?: boolean;
}

export interface SubtitleItem {
  url: string;
  language: string;
  format: string;
  is_default?: boolean;
}

export interface SkipTimes {
  intro_start: number | null;
  intro_end: number | null;
  outro_start: number | null;
  outro_end: number | null;
}

export interface SourcesResult {
  streams: StreamItem[];
  subtitles: SubtitleItem[];
  skip: SkipTimes | null;
}

export interface EpisodeItem {
  number: number;
  pipe_id: string;
  title: string | null;
  image: string | null;
  synopsis?: string | null;
  filler: boolean;
}

export interface ProviderEpisodes {
  sub: EpisodeItem[];
  dub: EpisodeItem[];
}

export type EpisodesResult = Record<string, ProviderEpisodes>;

export interface HistoryEntry {
  anilist_id: number;
  title: string;
  cover: string | null;
  episode_number: number;
  episode_title: string | null;
  provider: string;
  category: 'sub' | 'dub' | string;
  position_ms: number;
  duration_ms: number;
  updated_at: number;
  from_remote: boolean;
}

export interface WatchlistEntry {
  anilist_id: number;
  title: string;
  cover: string | null;
  format: string | null;
  average_score: number | null;
  added_at: number;
}

export interface MediaListEntry {
  id: number;
  progress: number;
  score: number;
  status: string | null;
  title: string | null;
  cover: string | null;
  format_str: string | null;
  total_episodes: number | null;
  average_score: number | null;
}

export interface Viewer {
  id: number;
  name: string;
  avatar_url: string | null;
  anime_count: number;
  episodes_watched: number;
  minutes_watched: number;
  mean_score: number;
}

export interface AiringEntry {
  id: number;
  airing_at: number;
  episode: number;
  media_id: number;
  media_title: string;
  cover_image: string | null;
  banner_image: string | null;
  format: string | null;
  duration: number | null;
  average_score: number | null;
}

export interface NotificationPreference {
  media_id: number;
  enabled: boolean;
  media_title: string;
  cover_image: string | null;
}

export interface DownloadRecord {
  id: string;
  anilist_id: number;
  episode_num: number;
  episode_title: string | null;
  series_title: string;
  series_cover: string | null;
  provider: string;
  category: string;
  quality: string;
  status: 'QUEUED' | 'DOWNLOADING' | 'COMPLETED' | 'FAILED' | 'PAUSED' | string;
  progress: number;
  file_path: string | null;
  file_size: number | null;
  duration_s: number | null;
  error_msg: string | null;
  created_at: number;
  updated_at: number;
}

export interface StorageCheck {
  ok: boolean;
  free_bytes: number;
  needed_bytes: number;
}

export interface UpdateInfo {
  version: string;
  changelog: string;
  published_at: string;
  release_url: string;
}

export interface HomeData {
  trending: Media[];
  popular: Media[];
  top_rated: Media[];
  newest: Media[];
  continue_watching: HistoryEntry[];
}

export interface AppSettings {
  autoplay: boolean;
  auto_skip_intro_outro: boolean;
  prefer_dub: boolean;
  subtitles_with_dub: boolean;
  default_quality: string;
  player_gestures: boolean;
  caption_text_scale: number;
  caption_text_color: string;
  caption_background_color: string;
  caption_background_opacity: number;
  caption_bold_text: boolean;
  caption_bottom_margin: number;
  caption_edge_style: string;
  persistent_caption_delays: string;
  download_quality: string;
  download_destination: string;
  download_dir: string;
  hide_adult_content: boolean;
  blur_episode_thumbnails: boolean;
  auto_sync_anilist: boolean;
  sync_watchlist_to_anilist: boolean;
  release_notifications: boolean;
  episode_layout: string;
  sidebar_expanded: boolean;
  menu_language: string;
  preferred_provider: string;
  server_priority: string;
  last_pipe_origin: string;
  enable_adult_providers: boolean;
  update_check_on_launch: boolean;
  enable_doh?: boolean;
  doh_provider?: 'cloudflare' | 'google' | 'adguard' | string;
  anilist_token?: string | null;
  anilist_viewer_id?: number | null;
  mal_access_token?: string | null;
  mal_refresh_token?: string | null;
  mal_token_expires_at?: number | null;
  external_player?: 'builtin' | 'mpv' | 'vlc';
}

export interface AnimeThemeEntry {
  theme_type: string; // "OP" or "ED"
  sequence: number | null;
  slug: string;
  song_title: string | null;
  artist_name: string | null;
  episodes: string | null;
  video_url: string | null;
  audio_url: string | null;
  resolution: number | null;
}

export interface LocalAnimeFile {
  file_path: string;
  file_name: string;
  file_size: number;
  parsed_title: string;
  parsed_episode: number | null;
}

export interface CustomPlaylist {
  id: string;
  name: string;
  description: string | null;
  created_at: number;
  updated_at: number;
  item_count: number;
  covers: string[];
}

export interface CustomPlaylistItem {
  id: string;
  playlist_id: string;
  anilist_id: number;
  mal_id: number | null;
  episode_num: number;
  episode_title: string | null;
  series_title: string;
  series_cover: string | null;
  category: string;
  sort_order: number;
  added_at: number;
}

export interface ExternalPlaylistItem {
  url: string;
  title: string;
  referer?: string | null;
}
