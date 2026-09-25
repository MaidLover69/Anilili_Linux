import React from 'react';
import { PlayerPlaylistDrawer } from './PlayerPlaylistDrawer';
import type { EpisodeItem } from '../types';

export interface InPlayerPlaylistProps {
  isOpen: boolean;
  onClose: () => void;
  episodes: EpisodeItem[];
  currentEpisodeNumber: number;
  onSelectEpisode: (epNum: number) => void;
  queuedEpisodes?: number[];
  onToggleQueueEpisode?: (epNum: number) => void;
  onPlayQueue?: () => void;
  onClearQueue?: () => void;
}

export const InPlayerPlaylist: React.FC<InPlayerPlaylistProps> = ({
  isOpen,
  onClose,
  episodes,
  currentEpisodeNumber,
  onSelectEpisode,
  queuedEpisodes = [],
  onToggleQueueEpisode = () => {},
  onPlayQueue = () => {},
  onClearQueue = () => {},
}) => {
  return (
    <PlayerPlaylistDrawer
      isOpen={isOpen}
      onClose={onClose}
      episodes={episodes}
      currentEpisodeNumber={currentEpisodeNumber}
      onSelectEpisode={onSelectEpisode}
      queuedEpisodes={queuedEpisodes}
      onToggleQueueEpisode={onToggleQueueEpisode}
      onPlayQueue={onPlayQueue}
      onClearQueue={onClearQueue}
    />
  );
};

export default InPlayerPlaylist;
