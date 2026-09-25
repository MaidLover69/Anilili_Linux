import React from 'react';
import { Dialog } from './Dialog';
import { useUIStore } from '../stores/uiStore';
import { useCheckUpdate } from '../hooks';
import { Sparkles, Download, ExternalLink } from 'lucide-react';

export const UpdateDialog: React.FC = () => {
  const { updateDialogOpen, setUpdateDialogOpen } = useUIStore();
  const { data: updateInfo } = useCheckUpdate();

  if (!updateInfo) return null;

  return (
    <Dialog
      isOpen={updateDialogOpen}
      onClose={() => setUpdateDialogOpen(false)}
      title="Update Available"
      maxWidth="max-w-md"
    >
      <div className="flex flex-col gap-4">
        <div className="flex items-center gap-3 p-3 rounded-xl bg-accent-tint border border-border-focus">
          <div className="p-2 rounded-lg bg-accent-500 text-white">
            <Sparkles className="w-5 h-5" />
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-bold text-accent-300">
              Anilili v{updateInfo.version}
            </span>
            <span className="text-[11px] text-text-muted">
              Released: {new Date(updateInfo.published_at).toLocaleDateString()}
            </span>
          </div>
        </div>

        {/* Changelog */}
        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-semibold text-text-secondary">Release Notes</label>
          <div className="p-3 rounded-xl bg-bg-elevated border border-border text-xs text-text-muted max-h-48 overflow-y-auto whitespace-pre-wrap leading-relaxed">
            {updateInfo.changelog || 'Performance improvements and bug fixes.'}
          </div>
        </div>

        {/* Buttons */}
        <div className="flex items-center gap-3 mt-2">
          <button
            onClick={() => setUpdateDialogOpen(false)}
            className="flex-1 py-2.5 rounded-xl bg-bg-elevated hover:bg-bg-input text-text-secondary text-xs font-semibold border border-border transition-colors"
          >
            Later
          </button>
          <a
            href={updateInfo.release_url}
            target="_blank"
            rel="noreferrer"
            className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl bg-accent-500 hover:bg-accent-700 text-white text-xs font-bold shadow-lg shadow-accent-900/40 transition-all"
          >
            <Download className="w-3.5 h-3.5" />
            <span>Download Release</span>
          </a>
        </div>
      </div>
    </Dialog>
  );
};
