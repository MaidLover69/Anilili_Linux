import React, { useState, useEffect } from 'react';
import {
  X,
  Sliders,
  Type,
  FastForward,
  PlaySquare,
  MessageSquare,
  Volume2,
  Check,
  RotateCcw,
} from 'lucide-react';
import { usePlayerStore } from '../stores/playerStore';
import type { SubtitleItem } from '../types';

interface AudioTrackOption {
  id: number;
  name: string;
  lang: string;
}

interface SubtitleSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  subtitles?: SubtitleItem[];
  selectedSubtitleUrl?: string;
  onSelectSubtitle?: (url: string) => void;
  audioTracks?: AudioTrackOption[];
  selectedAudioTrackId?: number;
  onSelectAudioTrack?: (id: number) => void;
  category?: 'sub' | 'dub';
  onToggleCategory?: (cat: 'sub' | 'dub') => void;
}

const COLOR_OPTIONS = [
  { name: 'White', value: '#ffffff' },
  { name: 'Amber', value: '#facc15' },
  { name: 'Cyan', value: '#22d3ee' },
  { name: 'Lime', value: '#4ade80' },
  { name: 'Magenta', value: '#f43f5e' },
];

const BG_OPTIONS = [
  { name: 'Transparent', value: 'transparent' },
  { name: 'Subtle Black', value: 'rgba(0, 0, 0, 0.5)' },
  { name: 'Solid Black', value: '#000000' },
];

export const SubtitleSettingsModal: React.FC<SubtitleSettingsModalProps> = ({
  isOpen,
  onClose,
  subtitles = [],
  selectedSubtitleUrl = 'off',
  onSelectSubtitle,
  audioTracks = [],
  selectedAudioTrackId = -1,
  onSelectAudioTrack,
  category,
  onToggleCategory,
}) => {
  const [activeTab, setActiveTab] = useState<'tracks' | 'style' | 'playback'>('tracks');

  const {
    autoSkipIntroOutro,
    setAutoSkipIntroOutro,
    autoPlayNext,
    setAutoPlayNext,
    subtitleFontSize,
    setSubtitleFontSize,
    subtitleColor,
    setSubtitleColor,
    subtitleBackground,
    setSubtitleBackground,
    subtitleDelayMs,
    setSubtitleDelayMs,
  } = usePlayerStore();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
    }
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-fade-in"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg bg-bg-card border border-border rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border px-6 py-4 bg-bg-surface/50">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-xl bg-accent-500/20 text-accent-300 border border-accent-500/30">
              <Sliders className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-base font-bold text-text-primary">
                Subtitle & Audio Customization
              </h3>
              <p className="text-[11px] text-text-muted">
                Track selection, typography styles, and sync offsets
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-text-secondary hover:text-white hover:bg-white/10 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex border-b border-border px-6 pt-3 bg-bg-surface/30 gap-2">
          <button
            onClick={() => setActiveTab('tracks')}
            className={`flex items-center gap-1.5 pb-2.5 px-3 text-xs font-bold border-b-2 transition-all cursor-pointer ${
              activeTab === 'tracks'
                ? 'border-accent-500 text-accent-300'
                : 'border-transparent text-text-muted hover:text-text-primary'
            }`}
          >
            <MessageSquare className="w-3.5 h-3.5" />
            <span>Tracks ({subtitles.length})</span>
          </button>

          <button
            onClick={() => setActiveTab('style')}
            className={`flex items-center gap-1.5 pb-2.5 px-3 text-xs font-bold border-b-2 transition-all cursor-pointer ${
              activeTab === 'style'
                ? 'border-accent-500 text-accent-300'
                : 'border-transparent text-text-muted hover:text-text-primary'
            }`}
          >
            <Type className="w-3.5 h-3.5" />
            <span>Appearance & Sync</span>
          </button>

          <button
            onClick={() => setActiveTab('playback')}
            className={`flex items-center gap-1.5 pb-2.5 px-3 text-xs font-bold border-b-2 transition-all cursor-pointer ${
              activeTab === 'playback'
                ? 'border-accent-500 text-accent-300'
                : 'border-transparent text-text-muted hover:text-text-primary'
            }`}
          >
            <PlaySquare className="w-3.5 h-3.5" />
            <span>Playback Controls</span>
          </button>
        </div>

        {/* Content Area */}
        <div className="p-6 flex flex-col gap-5 overflow-y-auto max-h-[60vh]">
          {/* TAB 1: TRACKS (Subtitles & Audio) */}
          {activeTab === 'tracks' && (
            <div className="flex flex-col gap-5">
              {/* Subtitle Track Selector */}
              <div className="flex flex-col gap-2">
                <span className="text-xs font-bold text-accent-300 uppercase tracking-wider">
                  Subtitle Track
                </span>
                <div className="grid grid-cols-1 gap-1.5 max-h-48 overflow-y-auto pr-1">
                  {/* Off Option */}
                  <button
                    onClick={() => onSelectSubtitle && onSelectSubtitle('off')}
                    className={`flex items-center justify-between p-3 rounded-xl border text-xs font-bold transition-all cursor-pointer ${
                      selectedSubtitleUrl === 'off'
                        ? 'border-accent-500 bg-accent-500/20 text-white shadow-sm'
                        : 'border-border bg-bg-surface text-text-secondary hover:text-white hover:bg-bg-elevated'
                    }`}
                  >
                    <span>Off (No Subtitles)</span>
                    {selectedSubtitleUrl === 'off' && <Check className="w-4 h-4 text-accent-300" />}
                  </button>

                  {/* Available Subtitle Tracks */}
                  {subtitles.map((sub, idx) => {
                    const isSelected = selectedSubtitleUrl === sub.url;
                    return (
                      <button
                        key={idx}
                        onClick={() => onSelectSubtitle && onSelectSubtitle(sub.url)}
                        className={`flex items-center justify-between p-3 rounded-xl border text-xs font-bold transition-all cursor-pointer ${
                          isSelected
                            ? 'border-accent-500 bg-accent-500/20 text-white shadow-sm'
                            : 'border-border bg-bg-surface text-text-secondary hover:text-white hover:bg-bg-elevated'
                        }`}
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          <span className="px-2 py-0.5 rounded bg-accent-500/20 text-accent-300 text-[10px] font-extrabold uppercase">
                            {sub.format || 'VTT'}
                          </span>
                          <span className="truncate">{sub.language || `Track ${idx + 1}`}</span>
                          {sub.is_default && (
                            <span className="text-[10px] text-text-muted">(Default)</span>
                          )}
                        </div>
                        {isSelected && <Check className="w-4 h-4 text-accent-300" />}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Audio Tracks Selector */}
              <div className="flex flex-col gap-2 pt-3 border-t border-border">
                <span className="text-xs font-bold text-accent-300 uppercase tracking-wider flex items-center gap-1.5">
                  <Volume2 className="w-3.5 h-3.5" /> Audio Stream / Dubbing
                </span>

                {audioTracks.length > 0 ? (
                  <div className="grid grid-cols-1 gap-1.5">
                    {audioTracks.map((trk) => {
                      const isSelected = selectedAudioTrackId === trk.id;
                      return (
                        <button
                          key={trk.id}
                          onClick={() => onSelectAudioTrack && onSelectAudioTrack(trk.id)}
                          className={`flex items-center justify-between p-3 rounded-xl border text-xs font-bold transition-all cursor-pointer ${
                            isSelected
                              ? 'border-accent-500 bg-accent-500/20 text-white shadow-sm'
                              : 'border-border bg-bg-surface text-text-secondary hover:text-white hover:bg-bg-elevated'
                          }`}
                        >
                          <span>{trk.name || `Audio Track ${trk.id + 1}`}</span>
                          {isSelected && <Check className="w-4 h-4 text-accent-300" />}
                        </button>
                      );
                    })}
                  </div>
                ) : (
                  <div className="flex items-center justify-between p-3 rounded-xl bg-bg-surface border border-border">
                    <span className="text-xs text-text-secondary">Audio Mode</span>
                    {onToggleCategory && category && (
                      <div className="flex items-center gap-1 bg-bg-card p-1 rounded-lg border border-border">
                        <button
                          onClick={() => onToggleCategory('sub')}
                          className={`px-3 py-1 rounded-md text-xs font-bold transition-all ${
                            category === 'sub'
                              ? 'bg-accent-500 text-white'
                              : 'text-text-muted hover:text-white'
                          }`}
                        >
                          Original (Sub)
                        </button>
                        <button
                          onClick={() => onToggleCategory('dub')}
                          className={`px-3 py-1 rounded-md text-xs font-bold transition-all ${
                            category === 'dub'
                              ? 'bg-accent-500 text-white'
                              : 'text-text-muted hover:text-white'
                          }`}
                        >
                          English Dub
                        </button>
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 2: STYLE & TIMING */}
          {activeTab === 'style' && (
            <div className="flex flex-col gap-5">
              {/* Subtitle Font Size */}
              <div className="flex flex-col gap-2">
                <div className="flex items-center justify-between text-xs">
                  <span className="font-semibold text-text-secondary flex items-center gap-1.5">
                    <Type className="w-3.5 h-3.5" /> Subtitle Font Size
                  </span>
                  <span className="font-bold text-accent-300">{subtitleFontSize}px</span>
                </div>
                <input
                  type="range"
                  min={12}
                  max={40}
                  step={1}
                  value={subtitleFontSize}
                  onChange={(e) => setSubtitleFontSize(parseInt(e.target.value, 10))}
                  className="w-full h-1.5 bg-bg-surface rounded-lg appearance-none cursor-pointer accent-accent-500"
                />
              </div>

              {/* Subtitle Color Options */}
              <div className="flex flex-col gap-2">
                <span className="text-xs font-semibold text-text-secondary">
                  Subtitle Text Color
                </span>
                <div className="grid grid-cols-5 gap-2">
                  {COLOR_OPTIONS.map((c) => (
                    <button
                      key={c.value}
                      onClick={() => setSubtitleColor(c.value)}
                      className={`py-1.5 px-2 rounded-lg text-xs font-bold border transition-all cursor-pointer ${
                        subtitleColor === c.value
                          ? 'border-accent-400 bg-accent-500/20 text-white shadow-sm'
                          : 'border-border bg-bg-surface text-text-secondary hover:text-white'
                      }`}
                    >
                      <div className="flex items-center justify-center gap-1.5">
                        <span
                          className="w-2.5 h-2.5 rounded-full border border-black/40"
                          style={{ backgroundColor: c.value }}
                        />
                        <span>{c.name}</span>
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Subtitle Background */}
              <div className="flex flex-col gap-2">
                <span className="text-xs font-semibold text-text-secondary">
                  Subtitle Background Styling
                </span>
                <div className="grid grid-cols-3 gap-2">
                  {BG_OPTIONS.map((bg) => (
                    <button
                      key={bg.value}
                      onClick={() => setSubtitleBackground(bg.value)}
                      className={`py-1.5 px-2 rounded-lg text-xs font-bold border transition-all cursor-pointer ${
                        subtitleBackground === bg.value
                          ? 'border-accent-400 bg-accent-500/20 text-white shadow-sm'
                          : 'border-border bg-bg-surface text-text-secondary hover:text-white'
                      }`}
                    >
                      {bg.name}
                    </button>
                  ))}
                </div>
              </div>

              {/* Subtitle Delay Sync */}
              <div className="flex flex-col gap-2 pt-2 border-t border-border">
                <div className="flex items-center justify-between text-xs">
                  <span className="font-semibold text-text-secondary">
                    Subtitle Timing Sync Offset
                  </span>
                  <span className="font-bold text-accent-300">
                    {subtitleDelayMs > 0 ? `+${(subtitleDelayMs / 1000).toFixed(2)}s` : `${(subtitleDelayMs / 1000).toFixed(2)}s`}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setSubtitleDelayMs(subtitleDelayMs - 250)}
                    className="px-2.5 py-1 rounded-lg bg-bg-surface border border-border text-xs font-bold text-text-secondary hover:text-white transition-colors cursor-pointer"
                  >
                    -0.25s
                  </button>
                  <input
                    type="range"
                    min={-5000}
                    max={5000}
                    step={100}
                    value={subtitleDelayMs}
                    onChange={(e) => setSubtitleDelayMs(parseInt(e.target.value, 10))}
                    className="flex-1 h-1.5 bg-bg-surface rounded-lg appearance-none cursor-pointer accent-accent-500"
                  />
                  <button
                    onClick={() => setSubtitleDelayMs(subtitleDelayMs + 250)}
                    className="px-2.5 py-1 rounded-lg bg-bg-surface border border-border text-xs font-bold text-text-secondary hover:text-white transition-colors cursor-pointer"
                  >
                    +0.25s
                  </button>
                  {subtitleDelayMs !== 0 && (
                    <button
                      onClick={() => setSubtitleDelayMs(0)}
                      className="p-1.5 rounded-lg bg-red-500/20 text-red-300 border border-red-500/30 text-xs font-bold cursor-pointer"
                      title="Reset delay"
                    >
                      <RotateCcw className="w-3.5 h-3.5" />
                    </button>
                  )}
                </div>
              </div>

              {/* Live Preview Box */}
              <div className="p-4 rounded-xl bg-black/90 border border-border/80 flex items-center justify-center min-h-[60px]">
                <span
                  style={{
                    fontSize: `${subtitleFontSize}px`,
                    color: subtitleColor,
                    backgroundColor: subtitleBackground,
                    padding: '2px 8px',
                    borderRadius: '4px',
                    textShadow: '0 0 4px #000, 0 0 8px #000',
                  }}
                  className="font-semibold text-center select-none"
                >
                  Subtitle Preview • Subtitle Timing Offset
                </span>
              </div>
            </div>
          )}

          {/* TAB 3: PLAYBACK */}
          {activeTab === 'playback' && (
            <div className="flex flex-col gap-3">
              <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-surface border border-border">
                <div className="flex items-center gap-2.5">
                  <FastForward className="w-4 h-4 text-accent-300" />
                  <div className="flex flex-col">
                    <span className="text-xs font-bold text-text-primary">
                      Auto-Skip Intro & Outro (AniSkip)
                    </span>
                    <span className="text-[11px] text-text-muted">
                      Automatically jump past opening and ending intervals
                    </span>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={autoSkipIntroOutro}
                  onChange={(e) => setAutoSkipIntroOutro(e.target.checked)}
                  className="w-4 h-4 accent-accent-500 rounded cursor-pointer"
                />
              </div>

              <div className="flex items-center justify-between p-3.5 rounded-xl bg-bg-surface border border-border">
                <div className="flex items-center gap-2.5">
                  <PlaySquare className="w-4 h-4 text-accent-300" />
                  <div className="flex flex-col">
                    <span className="text-xs font-bold text-text-primary">
                      Auto-Play Next Episode
                    </span>
                    <span className="text-[11px] text-text-muted">
                      Automatically start the next episode upon completion
                    </span>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={autoPlayNext}
                  onChange={(e) => setAutoPlayNext(e.target.checked)}
                  className="w-4 h-4 accent-accent-500 rounded cursor-pointer"
                />
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-t border-border p-4 bg-bg-surface/50">
          <button
            onClick={onClose}
            className="w-full py-2.5 rounded-xl bg-accent-500 hover:bg-accent-600 text-white font-bold text-xs transition-colors shadow-lg cursor-pointer"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
};
