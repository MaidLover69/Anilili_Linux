import React, { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const TitleBar: React.FC = () => {
  const [isFullscreen, setIsFullscreen] = useState(false);

  useEffect(() => {
    const handler = () => setIsFullscreen(!!document.fullscreenElement);
    document.addEventListener('fullscreenchange', handler);
    return () => document.removeEventListener('fullscreenchange', handler);
  }, []);

  if (isFullscreen) return null;

  const handleMinimize = () => {
    try {
      getCurrentWindow().minimize();
    } catch (_) {}
  };

  const handleToggleMaximize = () => {
    try {
      getCurrentWindow().toggleMaximize();
    } catch (_) {}
  };

  const handleClose = () => {
    try {
      getCurrentWindow().close();
    } catch (_) {}
  };

  return (
    <div
      data-tauri-drag-region
      className="h-8 w-full fixed top-0 left-0 z-50 flex items-center justify-between px-3 bg-[#050506] border-b border-white/5 select-none"
      style={{ WebkitAppRegion: 'drag' } as React.CSSProperties}
    >
      <div className="flex items-center gap-2 pointer-events-none">
        <span className="text-xs text-[--text-muted] font-semibold tracking-wider">Anilili</span>
      </div>
      <div
        className="flex items-center gap-2"
        style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        <button
          type="button"
          onClick={handleMinimize}
          className="p-1 text-xs text-text-muted hover:text-white transition-colors"
          aria-label="Minimize"
        >
          ─
        </button>
        <button
          type="button"
          onClick={handleToggleMaximize}
          className="p-1 text-xs text-text-muted hover:text-white transition-colors"
          aria-label="Maximize"
        >
          □
        </button>
        <button
          type="button"
          onClick={handleClose}
          className="p-1 text-xs text-text-muted hover:text-red-400 transition-colors"
          aria-label="Close"
        >
          ×
        </button>
      </div>
    </div>
  );
};
