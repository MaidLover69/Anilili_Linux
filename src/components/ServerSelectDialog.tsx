import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Server,
  Play,
  CheckCircle2,
  X,
  Sparkles,
  Volume2,
  Mic,
  Zap,
} from 'lucide-react';
import { Dialog } from './Dialog';
import { EpisodeItem as EpisodeItemType, EpisodesResult } from '../types';

interface ServerSelectDialogProps {
  isOpen: boolean;
  onClose: () => void;
  anilistId: number;
  animeTitle: string;
  episode: EpisodeItemType | null;
  episodesData?: EpisodesResult;
  isLoading?: boolean;
}

export const ServerSelectDialog: React.FC<ServerSelectDialogProps> = ({
  isOpen,
  onClose,
  anilistId,
  animeTitle,
  episode,
  episodesData = {},
  isLoading = false,
}) => {
  const navigate = useNavigate();
  const [category, setCategory] = useState<'sub' | 'dub'>('sub');

  if (!episode) return null;

  // Extract available servers for this specific episode
  const availableServers = Object.keys(episodesData)
    .map((providerName) => {
      const pData = episodesData[providerName];
      const subAvailable = pData?.sub?.some(
        (ep) => Math.abs(ep.number - episode.number) < 0.01
      );
      const dubAvailable = pData?.dub?.some(
        (ep) => Math.abs(ep.number - episode.number) < 0.01
      );

      return {
        name: providerName,
        subAvailable,
        dubAvailable,
      };
    })
    .filter((s) => (category === 'sub' ? s.subAvailable : s.dubAvailable));

  const handleLaunchServer = (providerName: string) => {
    onClose();
    navigate(
      `/watch/${anilistId}/${episode.number}?cat=${category}&prov=${encodeURIComponent(
        providerName
      )}`
    );
  };

  return (
    <Dialog isOpen={isOpen} onClose={onClose} title={`Select Server • Episode ${episode.number}`}>
      <div className="flex flex-col gap-5 select-none">
        {/* Header Info */}
        <div className="flex items-center justify-between p-3 rounded-xl bg-bg-elevated border border-border">
          <div className="flex flex-col min-w-0 pr-2">
            <span className="text-xs font-bold text-white truncate">{animeTitle}</span>
            <span className="text-[11px] text-text-muted mt-0.5">
              {episode.title || `Episode ${episode.number}`}
            </span>
          </div>

          {/* Sub / Dub selector */}
          <div className="flex items-center p-1 rounded-lg bg-bg-card border border-border flex-shrink-0">
            <button
              type="button"
              onClick={() => setCategory('sub')}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-bold transition-all ${
                category === 'sub'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <Volume2 className="w-3 h-3" />
              <span>SUB</span>
            </button>
            <button
              type="button"
              onClick={() => setCategory('dub')}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-bold transition-all ${
                category === 'dub'
                  ? 'bg-accent-500 text-white shadow-sm'
                  : 'text-text-secondary hover:text-white'
              }`}
            >
              <Mic className="w-3 h-3" />
              <span>DUB</span>
            </button>
          </div>
        </div>

        {/* Server List */}
        <div className="flex flex-col gap-2 max-h-64 overflow-y-auto pr-1">
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <div className="w-8 h-8 rounded-full border-2 border-accent-500 border-t-transparent animate-spin mb-3" />
              <span className="text-xs font-semibold text-text-primary">
                Resolving streaming servers, please wait...
              </span>
            </div>
          ) : availableServers.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-center bg-bg-elevated/50 rounded-xl border border-border">
              <Server className="w-8 h-8 text-text-muted mb-2 opacity-50" />
              <span className="text-xs font-semibold text-text-primary">
                No active servers found for {category.toUpperCase()}
              </span>
              <span className="text-[10px] text-text-muted mt-1">
                Try switching audio between SUB and DUB.
              </span>
            </div>
          ) : (
            availableServers.map((server, idx) => (
              <button
                key={server.name}
                type="button"
                onClick={() => handleLaunchServer(server.name)}
                className="group flex items-center justify-between p-3 rounded-xl bg-bg-elevated hover:bg-bg-input border border-border hover:border-border-focus transition-all duration-200 cursor-pointer text-left"
              >
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-accent-tint text-accent-300 border border-border-focus">
                    <Server className="w-4 h-4" />
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs font-bold text-white group-hover:text-accent-300 transition-colors">
                        {server.name}
                      </span>
                      {idx === 0 && (
                        <span className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-300 text-[9px] font-bold border border-emerald-500/30">
                          <Zap className="w-2.5 h-2.5" /> Recommended
                        </span>
                      )}
                    </div>
                    <span className="text-[10px] text-text-muted">
                      HD / Multi-Quality • Adaptive HLS
                    </span>
                  </div>
                </div>

                {(() => {
                  const miruroMirrors = ['bonk', 'kiwi', 'pewe', 'bee', 'ally', 'moo', 'hop', 'nun', 'bun', 'twin', 'cog', 'telli'];
                  const isMiruro = miruroMirrors.includes(server.name.toLowerCase());
                  const subTag = isMiruro ? 's-sub' : 'h-sub';
                  return (
                    <div className="flex items-center gap-2">
                      <span className={`px-2 py-0.5 rounded text-[10px] font-extrabold uppercase tracking-wider border ${
                        subTag === 's-sub'
                          ? 'bg-purple-500/20 text-purple-300 border-purple-500/40'
                          : 'bg-amber-500/20 text-amber-300 border-amber-500/40'
                      }`}>
                        {subTag}
                      </span>
                      <span className="px-2 py-0.5 rounded text-[10px] font-semibold bg-white/5 text-text-secondary">
                        {category.toUpperCase()}
                      </span>
                      <div className="p-1.5 rounded-full bg-accent-500 text-white transform group-hover:scale-110 transition-transform">
                        <Play className="w-3.5 h-3.5 fill-white ml-0.5" />
                      </div>
                    </div>
                  );
                })()}
              </button>
            ))
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 pt-2 border-t border-border">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-xl bg-bg-elevated hover:bg-bg-input text-text-secondary hover:text-white text-xs font-semibold border border-border transition-colors cursor-pointer"
          >
            Cancel
          </button>
        </div>
      </div>
    </Dialog>
  );
};

export default ServerSelectDialog;
