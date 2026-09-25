import React from 'react';
import { X, Keyboard, Play, Navigation, Settings } from 'lucide-react';

interface HotkeyCheatSheetProps {
  isOpen: boolean;
  onClose: () => void;
}

interface ShortcutGroup {
  title: string;
  icon: React.ReactNode;
  shortcuts: { keys: string[]; description: string }[];
}

const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: 'Video Player Controls',
    icon: <Play className="w-4 h-4 text-accent-500" />,
    shortcuts: [
      { keys: ['Space', 'K'], description: 'Play / Pause video' },
      { keys: ['←', '→'], description: 'Seek backward / forward 5s' },
      { keys: ['J', 'L'], description: 'Seek backward / forward 10s' },
      { keys: ['↑', '↓'], description: 'Volume up / down 5%' },
      { keys: ['M'], description: 'Toggle Mute' },
      { keys: ['F'], description: 'Toggle Fullscreen' },
      { keys: ['S'], description: 'Skip Intro / Outro (AniSkip)' },
      { keys: ['N'], description: 'Next Episode' },
      { keys: ['P'], description: 'Previous Episode' },
    ],
  },
  {
    title: 'App Navigation & Global',
    icon: <Navigation className="w-4 h-4 text-emerald-400" />,
    shortcuts: [
      { keys: ['Ctrl', 'K'], description: 'Open Global Command Palette & Search' },
      { keys: ['?'], description: 'Toggle this Keyboard Shortcuts Cheatsheet' },
      { keys: ['Esc'], description: 'Close modals / Return to previous view' },
      { keys: ['1' , '2', '3', '4'], description: 'Switch between Library tabs' },
    ],
  },
  {
    title: 'Advanced & Integrations',
    icon: <Settings className="w-4 h-4 text-amber-400" />,
    shortcuts: [
      { keys: ['D'], description: 'Toggle SUB / DUB category' },
      { keys: ['Ctrl', 'D'], description: 'Open Bulk Download dialog' },
    ],
  },
];

export const HotkeyCheatSheet: React.FC<HotkeyCheatSheetProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-fadeIn"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl bg-[#0e0e12] border border-white/10 rounded-2xl shadow-2xl overflow-hidden glass-modal"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-white/[0.02]">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-accent-500/10 flex items-center justify-center text-accent-500">
              <Keyboard className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-white">Keyboard Shortcuts</h2>
              <p className="text-xs text-white/40">Quick reference for all player and app hotkeys</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-white/40 hover:text-white hover:bg-white/10 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 max-h-[70vh] overflow-y-auto space-y-6">
          {SHORTCUT_GROUPS.map((group) => (
            <div key={group.title} className="space-y-2.5">
              <div className="flex items-center gap-2 text-xs font-semibold text-white/50 uppercase tracking-wider">
                {group.icon}
                {group.title}
              </div>
              <div className="grid grid-cols-1 gap-2 bg-white/[0.02] border border-white/5 rounded-xl p-3">
                {group.shortcuts.map((sc, i) => (
                  <div key={i} className="flex items-center justify-between text-sm py-1">
                    <span className="text-white/70">{sc.description}</span>
                    <div className="flex items-center gap-1">
                      {sc.keys.map((k) => (
                        <kbd
                          key={k}
                          className="px-2 py-0.5 text-xs font-mono text-white/90 bg-white/10 border border-white/15 rounded-md shadow-sm"
                        >
                          {k}
                        </kbd>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 bg-black/40 border-t border-white/5 flex items-center justify-between text-xs text-white/40">
          <span>Tip: Press <kbd className="px-1.5 py-0.5 bg-white/10 rounded">?</kbd> at any time to toggle this overlay</span>
          <button
            onClick={onClose}
            className="px-3 py-1 bg-white/10 hover:bg-white/20 text-white rounded-lg transition-colors text-xs font-medium"
          >
            Got it
          </button>
        </div>
      </div>
    </div>
  );
};

export default HotkeyCheatSheet;
