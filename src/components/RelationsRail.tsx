import { open } from '@tauri-apps/plugin-shell';
import React, { useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { ChevronLeft, ChevronRight, Layers, Star } from 'lucide-react';
import { ShimmerImage } from './ShimmerImage';
import type { RelationEdge } from '../types';

interface RelationsRailProps {
  relations?: RelationEdge[];
}

const formatRelationType = (type: string | null): string => {
  if (!type) return 'RELATED';
  return type.replace(/_/g, ' ').toUpperCase();
};

const getRelationBadgeColor = (type: string | null): string => {
  const upper = type?.toUpperCase() || '';
  if (upper.includes('SEQUEL')) return 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30';
  if (upper.includes('PREQUEL')) return 'bg-blue-500/20 text-blue-300 border-blue-500/30';
  if (upper.includes('SIDE')) return 'bg-purple-500/20 text-purple-300 border-purple-500/30';
  if (upper.includes('SPIN')) return 'bg-amber-500/20 text-amber-300 border-amber-500/30';
  if (upper.includes('ALTERNATIVE')) return 'bg-rose-500/20 text-rose-300 border-rose-500/30';
  return 'bg-white/10 text-text-secondary border-border';
};

export const RelationsRail: React.FC<RelationsRailProps> = ({ relations = [] }) => {
  const navigate = useNavigate();
  const scrollRef = useRef<HTMLDivElement | null>(null);

  if (!relations || relations.length === 0) return null;

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
          <Layers className="w-4 h-4 text-accent-400" />
          <h2 className="text-xl font-bold text-text-primary tracking-tight">
            Franchise & Relations
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
        {relations.map((edge, index) => {
          const item = edge.node;
          if (!item) return null;

          const title =
            item.title?.english ||
            item.title?.userPreferred ||
            item.title?.romaji ||
            'Anime';

          const cover =
            item.coverImage?.extraLarge ||
            item.coverImage?.large ||
            '';

          const formatUpper = item.format?.toUpperCase() || '';
            const isManga =
              formatUpper.includes('MANGA') ||
              formatUpper.includes('NOVEL') ||
              formatUpper.includes('ONE_SHOT') ||
              formatUpper.includes('COMIC') ||
              formatUpper.includes('DOUJINSHI') ||
              formatUpper.includes('MANHWA') ||
              formatUpper.includes('MANHUA');

            const handleCardClick = () => {
              if (isManga) {
                const targetUrl = item.idMal
                  ? `https://myanimelist.net/manga/${item.idMal}`
                  : `https://anilist.co/manga/${item.id}`;
                open(targetUrl).catch(() => window.open(targetUrl, '_blank'));
              } else {
                navigate(`/detail/${item.id}`);
              }
            };

            return (
              <div
                key={item.id || index}
                onClick={handleCardClick}
                className="flex-shrink-0 w-48 flex flex-col rounded-xl overflow-hidden bg-bg-card/80 border border-border/80 hover:border-accent-500/50 hover:bg-bg-elevated transition-all shadow-md group cursor-pointer snap-start"
              >
              <div className="relative w-full h-64 overflow-hidden bg-bg-surface">
                <ShimmerImage
                  src={cover}
                  alt={title}
                  className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-bg-card via-transparent to-transparent opacity-80" />

                {/* Relation Type Badge */}
                <div className="absolute top-2 left-2">
                  <span
                    className={`px-2 py-0.5 rounded-md border text-[10px] font-extrabold tracking-wider shadow-md backdrop-blur-md ${getRelationBadgeColor(
                      edge.relationType
                    )}`}
                  >
                    {formatRelationType(edge.relationType)}
                  </span>
                </div>

                {/* Score Pill */}
                {item.averageScore && (
                  <div className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-black/60 backdrop-blur-md text-[10px] font-bold text-accent-300 border border-white/10">
                    <Star className="w-2.5 h-2.5 fill-accent-300 text-accent-300" />
                    <span>{item.averageScore}%</span>
                  </div>
                )}

                {/* Format & Status bottom info */}
                <div className="absolute bottom-2 left-2 right-2 flex items-center justify-between text-[11px] text-text-secondary font-medium">
                  <span>{item.format || 'TV'}</span>
                  <span className="capitalize">{item.status?.toLowerCase().replace(/_/g, ' ') || ''}</span>
                </div>
              </div>

              <div className="p-3 flex flex-col gap-1">
                <span className="text-xs font-bold text-text-primary line-clamp-2 group-hover:text-accent-300 transition-colors">
                  {title}
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
