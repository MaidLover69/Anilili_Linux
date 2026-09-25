import React, { useState } from 'react';
import { Dialog } from './Dialog';
import { useUIStore } from '../stores/uiStore';
import { useEpisodes, useCheckStorage, useCreateDownload, useSettings } from '../hooks';
import { HardDrive, Download, AlertTriangle, CheckCircle } from 'lucide-react';
import type { Media } from '../types';

export const BulkDownloadDialog: React.FC = () => {
  const { bulkDownloadDialogOpen, closeBulkDownloadDialog, activeMediaForDownload, showToast } =
    useUIStore();
  const { data: settings } = useSettings();
  const media = activeMediaForDownload as Media | null;

  const [fromEp, setFromEp] = useState<number>(1);
  const [toEp, setToEp] = useState<number>(12);
  const [quality, setQuality] = useState<string>(settings?.download_quality || '720p');
  const [category, setCategory] = useState<'sub' | 'dub'>('sub');

  const { data: storage } = useCheckStorage(quality);
  const createDownloadMutation = useCreateDownload();

  if (!media) return null;

  const title =
    media.title.english ||
    media.title.user_preferred ||
    media.title.romaji ||
    'Anime';

  const totalEpisodes = media.episodes || 12;

  const handleStartBulkDownload = async () => {
    const start = Math.min(fromEp, toEp);
    const end = Math.max(fromEp, toEp);

    showToast({
      type: 'info',
      title: 'Queuing Downloads',
      message: `Queuing episodes ${start} to ${end} for "${title}"...`,
    });

    for (let epNum = start; epNum <= end; epNum++) {
      const id = `${media.id}-${epNum}-${category}-${quality}-${Date.now()}`;
      await createDownloadMutation.mutateAsync({
        id,
        anilist_id: media.id,
        episode_num: epNum,
        episode_title: `Episode ${epNum}`,
        series_title: title,
        series_cover: media.cover_image.extra_large || media.cover_image.large || null,
        provider: settings?.preferred_provider || 'auto',
        category,
        quality,
        status: 'QUEUED',
        progress: 0.0,
        file_path: null,
        file_size: null,
        duration_s: null,
        error_msg: null,
        created_at: Date.now(),
        updated_at: Date.now(),
      });
    }

    showToast({
      type: 'success',
      title: 'Bulk Download Started',
      message: `${end - start + 1} episodes queued for download.`,
    });
    closeBulkDownloadDialog();
  };

  return (
    <Dialog
      isOpen={bulkDownloadDialogOpen}
      onClose={closeBulkDownloadDialog}
      title="Bulk Episode Download"
      maxWidth="max-w-md"
    >
      <div className="flex flex-col gap-4">
        {/* Series info header */}
        <div className="flex items-center gap-3 p-3 rounded-xl bg-bg-elevated border border-border">
          <img
            src={media.cover_image.large || ''}
            alt={title}
            className="w-12 h-16 rounded-lg object-cover border border-border"
          />
          <div className="flex flex-col min-w-0">
            <h4 className="text-xs font-bold text-text-primary truncate">{title}</h4>
            <span className="text-[11px] text-text-muted mt-0.5">
              Total episodes available: {totalEpisodes}
            </span>
          </div>
        </div>

        {/* Range Selector */}
        <div className="flex flex-col gap-2">
          <label className="text-xs font-semibold text-text-secondary">Episode Range</label>
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 flex-1">
              <span className="text-xs text-text-muted">From:</span>
              <input
                type="number"
                min={1}
                max={totalEpisodes}
                value={fromEp}
                onChange={(e) => setFromEp(parseInt(e.target.value) || 1)}
                className="w-full px-3 py-1.5 rounded-lg bg-bg-input border border-border text-xs text-text-primary font-bold focus:border-border-focus outline-none"
              />
            </div>
            <div className="flex items-center gap-2 flex-1">
              <span className="text-xs text-text-muted">To:</span>
              <input
                type="number"
                min={1}
                max={totalEpisodes}
                value={toEp}
                onChange={(e) => setToEp(parseInt(e.target.value) || 1)}
                className="w-full px-3 py-1.5 rounded-lg bg-bg-input border border-border text-xs text-text-primary font-bold focus:border-border-focus outline-none"
              />
            </div>
          </div>
        </div>

        {/* Category and Quality */}
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold text-text-secondary">Audio</label>
            <select
              value={category}
              onChange={(e) => setCategory(e.target.value as 'sub' | 'dub')}
              className="px-3 py-2 rounded-lg bg-bg-input border border-border text-xs text-text-primary focus:border-border-focus outline-none"
            >
              <option value="sub">Subtitled (Japanese)</option>
              <option value="dub">Dubbed (English)</option>
            </select>
          </div>

          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold text-text-secondary">Quality</label>
            <select
              value={quality}
              onChange={(e) => setQuality(e.target.value)}
              className="px-3 py-2 rounded-lg bg-bg-input border border-border text-xs text-text-primary focus:border-border-focus outline-none"
            >
              <option value="1080p">1080p (FHD ~550MB/ep)</option>
              <option value="720p">720p (HD ~300MB/ep)</option>
              <option value="480p">480p (SD ~150MB/ep)</option>
            </select>
          </div>
        </div>

        {/* Disk Space Check Status */}
        {storage && (
          <div
            className={`flex items-center gap-2.5 p-3 rounded-lg border text-xs ${
              storage.ok
                ? 'bg-state-success/10 border-state-success/30 text-state-success'
                : 'bg-state-error/10 border-state-error/30 text-state-error'
            }`}
          >
            <HardDrive className="w-4 h-4 flex-shrink-0" />
            <span>
              Free Space: {(storage.free_bytes / (1024 * 1024 * 1024)).toFixed(1)} GB (Required headroom checked)
            </span>
          </div>
        )}

        {/* Action button */}
        <button
          onClick={handleStartBulkDownload}
          className="flex items-center justify-center gap-2 w-full py-3 rounded-xl bg-accent-500 hover:bg-accent-700 text-white text-xs font-bold shadow-lg shadow-accent-900/40 transition-all mt-2"
        >
          <Download className="w-4 h-4" />
          <span>Start Download ({Math.abs(toEp - fromEp) + 1} Episodes)</span>
        </button>
      </div>
    </Dialog>
  );
};
