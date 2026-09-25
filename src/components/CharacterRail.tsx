import { open } from '@tauri-apps/plugin-shell';
import React, { useRef } from 'react';
import { ChevronLeft, ChevronRight, Users } from 'lucide-react';
import { ShimmerImage } from './ShimmerImage';
import type { CharacterEdge } from '../types';

interface CharacterRailProps {
  characters?: CharacterEdge[];
}

export const CharacterRail: React.FC<CharacterRailProps> = ({ characters = [] }) => {
  const scrollRef = useRef<HTMLDivElement | null>(null);

  if (!characters || characters.length === 0) return null;

  const scroll = (direction: 'left' | 'right') => {
    if (scrollRef.current) {
      const scrollAmount = direction === 'left' ? -350 : 350;
      scrollRef.current.scrollBy({ left: scrollAmount, behavior: 'smooth' });
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-2">
          <Users className="w-4 h-4 text-accent-400" />
          <h2 className="text-xl font-bold text-text-primary tracking-tight">
            Characters & Voice Actors
          </h2>
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => scroll('left')}
            className="p-1.5 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-white hover:bg-bg-elevated transition-colors"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <button
            onClick={() => scroll('right')}
            className="p-1.5 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-white hover:bg-bg-elevated transition-colors"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div
        ref={scrollRef}
        className="flex gap-3.5 overflow-x-auto pb-3 pt-1 scrollbar-thin scrollbar-thumb-bg-elevated scrollbar-track-transparent snap-x"
        style={{ scrollbarWidth: 'thin' }}
      >
        {characters.map((edge, index) => {
          const char = edge.node;
          const va = edge.voiceActors?.[0];
          if (!char) return null;

          const isMain = edge.role?.toUpperCase() === 'MAIN';

          return (
            <div
              key={char.id || index}
              className="flex-shrink-0 w-72 flex items-center justify-between p-2.5 rounded-xl bg-bg-card/80 border border-border/80 hover:border-accent-500/40 hover:bg-bg-elevated/80 transition-all shadow-sm snap-start group"
            >
              {/* Character Info */}
              <div className="flex items-center gap-2.5 min-w-0 pr-2">
                <div className="w-12 h-16 rounded-lg overflow-hidden flex-shrink-0 border border-border/60">
                  <ShimmerImage
                    src={char.image?.large || char.image?.medium || ''}
                    alt={char.name?.full || 'Character'}
                    className="w-full h-full object-cover"
                  />
                </div>
                <div className="flex flex-col min-w-0">
                  <span className="text-xs font-bold text-text-primary truncate group-hover:text-accent-300 transition-colors">
                    {char.name?.full || 'Unknown'}
                  </span>
                  <span
                    className={`text-[10px] font-semibold tracking-wider uppercase mt-0.5 ${
                      isMain ? 'text-accent-400' : 'text-text-muted'
                    }`}
                  >
                    {edge.role || 'Supporting'}
                  </span>
                </div>
              </div>

              {/* Voice Actor Info */}
              {va && (
                <div
                  onClick={(e) => {
                    e.stopPropagation();
                    const url = va.id ? `https://myanimelist.net/people/${va.id}` : 'https://myanimelist.net';
                    open(url).catch(() => window.open(url, '_blank'));
                  }}
                  title={`Open ${va.name?.full || 'Voice Actor'} on MyAnimeList`}
                  className="flex items-center gap-2.5 min-w-0 pl-2 text-right border-l border-border/40 cursor-pointer group/va hover:opacity-90 transition-opacity"
                >
                  <div className="flex flex-col min-w-0">
                    <span className="text-xs font-bold text-text-secondary truncate group-hover/va:text-accent-300 transition-colors">
                      {va.name?.full || 'Voice Actor'}
                    </span>
                    <span className="text-[10px] text-text-muted mt-0.5">
                      Japanese
                    </span>
                  </div>
                  <div className="w-12 h-16 rounded-lg overflow-hidden flex-shrink-0 border border-border/60 group-hover/va:border-accent-500/60 transition-colors">
                    <ShimmerImage
                      src={va.image?.large || va.image?.medium || ''}
                      alt={va.name?.full || 'Voice Actor'}
                      className="w-full h-full object-cover"
                    />
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
