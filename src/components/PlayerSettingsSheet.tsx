import React from 'react';
import { X, Sliders, Check } from 'lucide-react';
import { CustomSelect } from './CustomSelect';

interface QualityLevel {
  id: number;
  height: number;
  name: string;
}

interface PlayerSettingsSheetProps {
  isOpen: boolean;
  onClose: () => void;
  qualityLevels: QualityLevel[];
  currentQualityLevel: number;
  onSelectQuality: (levelId: number) => void;
  playbackRate: number;
  onSelectPlaybackRate: (rate: number) => void;
}

export const PlayerSettingsSheet: React.FC<PlayerSettingsSheetProps> = ({
  isOpen,
  onClose,
  qualityLevels,
  currentQualityLevel,
  onSelectQuality,
  playbackRate,
  onSelectPlaybackRate,
}) => {
  if (!isOpen) return null;

  const qualityOptions = [
    { value: '-1', label: 'Auto (Best)' },
    ...qualityLevels.map((lvl) => ({
      value: lvl.id.toString(),
      label: lvl.name,
    })),
  ];

  const speedOptions = [
    { value: '0.5', label: '0.5x' },
    { value: '0.75', label: '0.75x' },
    { value: '1', label: '1.0x (Normal)' },
    { value: '1.25', label: '1.25x' },
    { value: '1.5', label: '1.5x' },
    { value: '2', label: '2.0x' },
    { value: '3', label: '3.0x' },
    { value: '4', label: '4.0x' },
  ];

  return (
    <div
      className="fixed inset-0 z-50 flex items-end justify-center bg-black/60 backdrop-blur-xs animate-fade-in p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md bg-[#141416] border border-white/15 rounded-2xl shadow-2xl overflow-hidden p-5 animate-slide-up flex flex-col gap-4 text-text-primary"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between pb-3 border-b border-white/10">
          <div className="flex items-center gap-2">
            <Sliders className="w-5 h-5 text-accent-400" />
            <h3 className="text-sm font-bold">Playback Settings</h3>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-lg text-text-secondary hover:text-white hover:bg-white/10 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Quality Selector (Opens Upward) */}
        <div className="flex items-center justify-between gap-4 py-1">
          <div className="flex flex-col">
            <span className="text-xs font-bold text-white">Video Quality</span>
            <span className="text-[10px] text-text-muted">Select preferred streaming resolution</span>
          </div>
          <CustomSelect
            value={currentQualityLevel.toString()}
            onChange={(val) => onSelectQuality(parseInt(val, 10))}
            options={qualityOptions}
            placement="up"
          />
        </div>

        {/* Speed Selector (Opens Upward) */}
        <div className="flex items-center justify-between gap-4 py-1">
          <div className="flex flex-col">
            <span className="text-xs font-bold text-white">Playback Speed</span>
            <span className="text-[10px] text-text-muted">Adjust video playback velocity</span>
          </div>
          <CustomSelect
            value={playbackRate.toString()}
            onChange={(val) => onSelectPlaybackRate(parseFloat(val))}
            options={speedOptions}
            placement="up"
          />
        </div>
      </div>
    </div>
  );
};

export default PlayerSettingsSheet;
