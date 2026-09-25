import React, { useState } from 'react';
import { Music, Play, Pause, Video, Volume2, X, Sparkles } from 'lucide-react';
import { useAnimeThemes } from '../hooks';
import type { AnimeThemeEntry } from '../types';

interface AnimeThemesPlayerProps {
  malId?: number | null;
  animeTitle: string;
}

export const AnimeThemesPlayer: React.FC<AnimeThemesPlayerProps> = ({ malId, animeTitle }) => {
  const { data: themes = [], isLoading } = useAnimeThemes(malId);
  const [activeMediaTheme, setActiveMediaTheme] = useState<AnimeThemeEntry | null>(null);
  const [mediaMode, setMediaMode] = useState<'video' | 'audio'>('video');

  if (isLoading || !themes || themes.length === 0) return null;

  const openings = themes.filter((t) => t.theme_type === 'OP');
  const endings = themes.filter((t) => t.theme_type === 'ED');

  return (
    <div className="flex flex-col gap-4 p-5 rounded-2xl bg-bg-card/80 border border-border shadow-md">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className="p-2 rounded-xl bg-accent-tint text-accent-300 border border-border-focus">
            <Music className="w-4 h-4" />
          </div>
          <div>
            <h3 className="text-base font-bold text-text-primary">
              Opening & Ending Soundtracks
            </h3>
            <p className="text-[11px] text-text-muted">
              Official Theme Songs from AnimeThemes.moe
            </p>
          </div>
        </div>
      </div>

      {/* Grid of Openings and Endings */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Openings */}
        {openings.length > 0 && (
          <div className="flex flex-col gap-2">
            <span className="text-xs font-bold text-accent-400 uppercase tracking-wider px-1">
              Openings (OP)
            </span>
            <div className="flex flex-col gap-2">
              {openings.map((op, idx) => (
                <div
                  key={idx}
                  className="flex items-center justify-between p-3 rounded-xl bg-bg-surface/80 border border-border hover:border-accent-500/40 hover:bg-bg-elevated/80 transition-all group"
                >
                  <div className="flex items-center gap-3 min-w-0 pr-2">
                    <span className="px-2 py-0.5 rounded-md bg-accent-500/20 text-accent-300 text-[11px] font-extrabold border border-accent-500/30">
                      {op.slug}
                    </span>
                    <div className="flex flex-col min-w-0">
                      <span className="text-xs font-bold text-text-primary truncate group-hover:text-accent-300 transition-colors">
                        {op.song_title || 'Theme Song'}
                      </span>
                      <span className="text-[11px] text-text-muted truncate">
                        {op.artist_name || 'Various Artists'}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 flex-shrink-0">
                    {op.video_url && (
                      <button
                        onClick={() => {
                          setActiveMediaTheme(op);
                          setMediaMode('video');
                        }}
                        title="Watch Music Video"
                        className="p-2 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-accent-300 hover:bg-bg-input transition-colors cursor-pointer"
                      >
                        <Video className="w-3.5 h-3.5" />
                      </button>
                    )}
                    {op.audio_url && (
                      <button
                        onClick={() => {
                          setActiveMediaTheme(op);
                          setMediaMode('audio');
                        }}
                        title="Listen to Audio"
                        className="p-2 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-accent-300 hover:bg-bg-input transition-colors cursor-pointer"
                      >
                        <Volume2 className="w-3.5 h-3.5" />
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Endings */}
        {endings.length > 0 && (
          <div className="flex flex-col gap-2">
            <span className="text-xs font-bold text-purple-400 uppercase tracking-wider px-1">
              Endings (ED)
            </span>
            <div className="flex flex-col gap-2">
              {endings.map((ed, idx) => (
                <div
                  key={idx}
                  className="flex items-center justify-between p-3 rounded-xl bg-bg-surface/80 border border-border hover:border-purple-500/40 hover:bg-bg-elevated/80 transition-all group"
                >
                  <div className="flex items-center gap-3 min-w-0 pr-2">
                    <span className="px-2 py-0.5 rounded-md bg-purple-500/20 text-purple-300 text-[11px] font-extrabold border border-purple-500/30">
                      {ed.slug}
                    </span>
                    <div className="flex flex-col min-w-0">
                      <span className="text-xs font-bold text-text-primary truncate group-hover:text-purple-300 transition-colors">
                        {ed.song_title || 'Theme Song'}
                      </span>
                      <span className="text-[11px] text-text-muted truncate">
                        {ed.artist_name || 'Various Artists'}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 flex-shrink-0">
                    {ed.video_url && (
                      <button
                        onClick={() => {
                          setActiveMediaTheme(ed);
                          setMediaMode('video');
                        }}
                        title="Watch Music Video"
                        className="p-2 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-purple-300 hover:bg-bg-input transition-colors cursor-pointer"
                      >
                        <Video className="w-3.5 h-3.5" />
                      </button>
                    )}
                    {ed.audio_url && (
                      <button
                        onClick={() => {
                          setActiveMediaTheme(ed);
                          setMediaMode('audio');
                        }}
                        title="Listen to Audio"
                        className="p-2 rounded-lg bg-bg-card border border-border text-text-secondary hover:text-purple-300 hover:bg-bg-input transition-colors cursor-pointer"
                      >
                        <Volume2 className="w-3.5 h-3.5" />
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Theme Media Player Modal */}
      {activeMediaTheme && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fade-in"
          onClick={() => setActiveMediaTheme(null)}
        >
          <div
            className="w-full max-w-3xl bg-bg-card border border-border rounded-2xl shadow-2xl overflow-hidden flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-border bg-bg-surface/60">
              <div className="flex items-center gap-2 min-w-0 pr-4">
                <span className="px-2 py-0.5 rounded-md bg-accent-500/20 text-accent-300 text-xs font-bold">
                  {activeMediaTheme.slug}
                </span>
                <div className="flex flex-col min-w-0">
                  <span className="text-sm font-bold text-text-primary truncate">
                    {activeMediaTheme.song_title || 'Theme Song'}
                  </span>
                  <span className="text-xs text-text-muted truncate">
                    {activeMediaTheme.artist_name || animeTitle}
                  </span>
                </div>
              </div>
              <button
                onClick={() => setActiveMediaTheme(null)}
                className="p-1.5 rounded-lg text-text-secondary hover:text-white hover:bg-white/10 transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Video/Audio Area */}
            {mediaMode === 'video' && activeMediaTheme.video_url ? (
              <div className="relative w-full aspect-video bg-black">
                <video
                  src={activeMediaTheme.video_url}
                  controls
                  autoPlay
                  className="w-full h-full object-contain"
                />
              </div>
            ) : (
              <div className="p-8 flex flex-col items-center justify-center gap-4 bg-bg-surface">
                <div className="w-20 h-20 rounded-full bg-accent-500/20 border border-accent-500/30 flex items-center justify-center text-accent-300 animate-pulse">
                  <Music className="w-10 h-10" />
                </div>
                <audio
                  src={activeMediaTheme.audio_url || activeMediaTheme.video_url || ''}
                  controls
                  autoPlay
                  className="w-full max-w-md"
                />
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
