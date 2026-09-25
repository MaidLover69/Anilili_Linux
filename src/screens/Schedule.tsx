import React, { useState, useMemo, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Calendar,
  Clock,
  Bell,
  BellOff,
  RotateCcw,
  AlertCircle,
  List,
  Grid,
  Menu,
} from 'lucide-react';
import { useAiringSchedule, useNotificationPreferences, useToggleNotificationPreference } from '../hooks';
import { ShimmerImage } from '../components/ShimmerImage';
import type { AiringEntry } from '../types';

const DAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

function formatCountdown(airingAt: number, currentNow: number): string {
  const diff = airingAt - currentNow;
  if (diff > 0) {
    const hours = Math.floor(diff / 3600);
    const minutes = Math.floor((diff % 3600) / 60);
    if (hours > 24) {
      const days = Math.floor(hours / 24);
      return `In ${days}d ${hours % 24}h`;
    }
    if (hours > 0) {
      return `In ${hours}h ${minutes}m`;
    }
    return `In ${minutes}m`;
  } else {
    const elapsed = Math.abs(diff);
    const hours = Math.floor(elapsed / 3600);
    const minutes = Math.floor((elapsed % 3600) / 60);
    if (hours > 24) {
      return 'Aired';
    }
    if (hours > 0) {
      return `Aired ${hours}h ago`;
    }
    return `Aired ${minutes}m ago`;
  }
}

export const Schedule: React.FC = () => {
  const navigate = useNavigate();
  const [selectedDayOffset, setSelectedDayOffset] = useState<number>(0); // 0 = today, +1 = tomorrow, etc.
  const [currentTime, setCurrentTime] = useState<number>(Math.floor(Date.now() / 1000));
  const [viewMode, setViewMode] = useState<'compact' | 'list' | 'grid'>('list');

  // Live countdown tick every 30s
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(Math.floor(Date.now() / 1000));
    }, 30000);
    return () => clearInterval(interval);
  }, []);

  // Stable midnight timestamp for today
  const todayMidnight = useMemo(() => {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    return Math.floor(d.getTime() / 1000);
  }, []);

  const startOfWeek = useMemo(() => todayMidnight - 86400, [todayMidnight]);
  const endOfWeek = useMemo(() => todayMidnight + 86400 * 8, [todayMidnight]);

  const {
    data: schedule = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useAiringSchedule(startOfWeek, endOfWeek);

  const { data: notificationPreferences = [] } = useNotificationPreferences();
  const toggleNotificationMutation = useToggleNotificationPreference();

  // Group entries by 7-day window
  const dayTabs = useMemo(() => {
    return Array.from({ length: 7 }).map((_, i) => {
      const d = new Date((todayMidnight + i * 86400) * 1000);
      const dayName = DAYS[d.getDay()];
      const isToday = i === 0;
      return {
        offset: i,
        label: isToday ? `Today (${dayName})` : dayName,
        dateString: d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' }),
        dayStart: todayMidnight + i * 86400,
        dayEnd: todayMidnight + (i + 1) * 86400 - 1,
      };
    });
  }, [todayMidnight]);

  const activeTab = dayTabs.find((t) => t.offset === selectedDayOffset) || dayTabs[0];

  const filteredEntries = useMemo(() => {
    return schedule
      .filter(
        (entry) => entry.airing_at >= activeTab.dayStart && entry.airing_at <= activeTab.dayEnd
      )
      .sort((a, b) => a.airing_at - b.airing_at);
  }, [schedule, activeTab]);

  const handleNotificationToggle = (entry: AiringEntry, e: React.MouseEvent) => {
    e.stopPropagation();
    const existing = notificationPreferences.find((p) => p.media_id === entry.media_id);
    const enabled = !(existing && existing.enabled);
    toggleNotificationMutation.mutate({
      media_id: entry.media_id,
      enabled,
      media_title: entry.media_title,
      cover_image: entry.cover_image,
    });
  };

  return (
    <div className="flex flex-col gap-6 pb-16 animate-fade-in select-none">
      {/* Top Controls Bar: 7-Day Tabs + View Mode Toggle */}
      <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-2xl bg-bg-card border border-border shadow-lg">
        {/* 7-Day Header Tabs */}
        <div className="flex items-center gap-2 overflow-x-auto no-scrollbar py-0.5">
          {dayTabs.map((tab) => (
            <button
              key={tab.offset}
              type="button"
              onClick={() => setSelectedDayOffset(tab.offset)}
              className={`flex flex-col items-center px-4 py-2 rounded-xl text-xs transition-all flex-shrink-0 cursor-pointer ${
                selectedDayOffset === tab.offset
                  ? 'bg-accent-500 text-white font-bold shadow-md shadow-accent-900/30'
                  : 'bg-bg-elevated text-text-secondary hover:text-white border border-border'
              }`}
            >
              <span className="font-semibold">{tab.label}</span>
              <span className="text-[10px] opacity-75 mt-0.5">{tab.dateString}</span>
            </button>
          ))}
        </div>

        {/* View Mode Toggle (Compact / List / Grid) */}
        <div className="flex items-center p-1 rounded-xl bg-bg-elevated border border-border flex-shrink-0">
          <button
            type="button"
            onClick={() => setViewMode('compact')}
            title="Compact Mode"
            className={`p-2 rounded-lg transition-all cursor-pointer ${
              viewMode === 'compact'
                ? 'bg-accent-500 text-white shadow-sm'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <Menu className="w-4 h-4" />
          </button>
          <button
            type="button"
            onClick={() => setViewMode('list')}
            title="List Mode"
            className={`p-2 rounded-lg transition-all cursor-pointer ${
              viewMode === 'list'
                ? 'bg-accent-500 text-white shadow-sm'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <List className="w-4 h-4" />
          </button>
          <button
            type="button"
            onClick={() => setViewMode('grid')}
            title="Grid Mode"
            className={`p-2 rounded-lg transition-all cursor-pointer ${
              viewMode === 'grid'
                ? 'bg-accent-500 text-white shadow-sm'
                : 'text-text-secondary hover:text-white'
            }`}
          >
            <Grid className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Error state with retry */}
      {isError && (
        <div className="flex flex-col items-center justify-center p-8 bg-red-950/20 border border-red-500/30 rounded-2xl text-center">
          <AlertCircle className="w-8 h-8 text-red-400 mb-2" />
          <h3 className="text-sm font-bold text-white">Failed to load airing schedule</h3>
          <p className="text-xs text-white/50 mt-1 max-w-md">
            {(error as any)?.message || 'AniList API request failed. Check your internet connection.'}
          </p>
          <button
            onClick={() => refetch()}
            className="mt-4 flex items-center gap-2 px-4 py-2 rounded-xl bg-accent-500 hover:bg-accent-700 text-white text-xs font-semibold shadow transition-colors cursor-pointer"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span>Retry</span>
          </button>
        </div>
      )}

      {/* Airing Schedule Content */}
      {isLoading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {Array.from({ length: 8 }).map((_, i) => (
            <div key={i} className="h-36 rounded-xl shimmer-placeholder border border-border" />
          ))}
        </div>
      ) : !isError && filteredEntries.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 bg-bg-card rounded-2xl border border-border text-center">
          <Calendar className="w-10 h-10 text-text-muted mb-3 opacity-30" />
          <h3 className="text-base font-bold text-text-primary">No Broadcasts Scheduled</h3>
          <p className="text-xs text-text-muted mt-1">
            No airing episodes found for {activeTab.label}.
          </p>
        </div>
      ) : viewMode === 'compact' ? (
        /* Compact Mode: High-density rows */
        <div className="flex flex-col gap-2">
          {filteredEntries.map((entry) => {
            const isNotified = notificationPreferences.some(
              (p) => p.media_id === entry.media_id && p.enabled
            );
            const airDate = new Date(entry.airing_at * 1000);
            const timeStr = airDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
            const countdownStr = formatCountdown(entry.airing_at, currentTime);
            const isUpcoming = entry.airing_at > currentTime;

            return (
              <div
                key={`${entry.id}-${entry.episode}`}
                onClick={() => navigate(`/detail/${entry.media_id}`)}
                className="group flex items-center justify-between px-4 py-2.5 rounded-xl bg-bg-card border border-border hover:border-border-focus hover:bg-bg-elevated cursor-pointer transition-all duration-200 shadow-sm"
              >
                <div className="flex items-center gap-3 min-w-0 flex-1">
                  <span className="flex items-center gap-1 text-[11px] font-bold text-accent-300 bg-accent-tint px-2 py-0.5 rounded border border-border-focus flex-shrink-0">
                    <Clock className="w-3 h-3" />
                    <span>{timeStr}</span>
                  </span>
                  <span className="text-xs font-bold text-text-primary group-hover:text-accent-300 truncate">
                    {entry.media_title}
                  </span>
                  <span className="text-[11px] font-semibold text-text-muted flex-shrink-0">
                    EP {entry.episode}
                  </span>
                </div>

                <div className="flex items-center gap-3 flex-shrink-0 ml-3">
                  <span
                    className={`px-2 py-0.5 rounded text-[10px] font-semibold ${
                      isUpcoming
                        ? 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
                        : 'bg-white/5 text-white/50'
                    }`}
                  >
                    {countdownStr}
                  </span>
                  <button
                    type="button"
                    onClick={(e) => handleNotificationToggle(entry, e)}
                    aria-label={isNotified ? 'Disable notification' : 'Enable notification'}
                    className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
                      isNotified
                        ? 'text-accent-300 bg-accent-tint border border-accent-500/40'
                        : 'text-text-muted hover:text-white hover:bg-bg-input'
                    }`}
                  >
                    {isNotified ? <Bell className="w-3.5 h-3.5 fill-current" /> : <BellOff className="w-3.5 h-3.5" />}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      ) : viewMode === 'grid' ? (
        /* Grid Mode: Poster cards */
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3.5">
          {filteredEntries.map((entry) => {
            const isNotified = notificationPreferences.some(
              (p) => p.media_id === entry.media_id && p.enabled
            );
            const airDate = new Date(entry.airing_at * 1000);
            const timeStr = airDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
            const countdownStr = formatCountdown(entry.airing_at, currentTime);
            const isUpcoming = entry.airing_at > currentTime;

            return (
              <div
                key={`${entry.id}-${entry.episode}`}
                onClick={() => navigate(`/detail/${entry.media_id}`)}
                className="group relative flex flex-col p-2.5 rounded-xl bg-bg-card border border-border hover:border-border-focus hover:bg-bg-elevated cursor-pointer transition-all duration-200 shadow card-hover-glow"
              >
                <div className="relative w-full aspect-[2/3] rounded-lg overflow-hidden bg-bg-elevated mb-2 border border-border">
                  <ShimmerImage
                    src={entry.cover_image}
                    alt={entry.media_title}
                    className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                  />
                  <div className="absolute top-1.5 left-1.5 flex items-center gap-1 text-[10px] font-bold text-white bg-black/75 px-1.5 py-0.5 rounded backdrop-blur-md">
                    <Clock className="w-2.5 h-2.5 text-accent-300" />
                    <span>{timeStr}</span>
                  </div>
                  <div className="absolute bottom-1.5 right-1.5 px-1.5 py-0.5 rounded bg-black/75 text-white text-[10px] font-bold backdrop-blur-md">
                    EP {entry.episode}
                  </div>
                </div>

                <h4 className="text-xs font-bold text-text-primary group-hover:text-accent-300 line-clamp-2 leading-snug">
                  {entry.media_title}
                </h4>

                <div className="mt-2 pt-1.5 border-t border-white/5 flex items-center justify-between">
                  <span
                    className={`px-1.5 py-0.5 rounded text-[10px] font-semibold ${
                      isUpcoming
                        ? 'bg-emerald-500/15 text-emerald-300'
                        : 'text-white/40'
                    }`}
                  >
                    {countdownStr}
                  </span>
                  <button
                    type="button"
                    onClick={(e) => handleNotificationToggle(entry, e)}
                    className={`p-1 rounded transition-colors ${
                      isNotified ? 'text-accent-300' : 'text-text-muted hover:text-white'
                    }`}
                  >
                    {isNotified ? <Bell className="w-3.5 h-3.5 fill-current" /> : <BellOff className="w-3.5 h-3.5" />}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        /* List Mode: Horizontal card layout */
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {filteredEntries.map((entry) => {
            const isNotified = notificationPreferences.some(
              (p) => p.media_id === entry.media_id && p.enabled
            );
            const airDate = new Date(entry.airing_at * 1000);
            const timeStr = airDate.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
            const countdownStr = formatCountdown(entry.airing_at, currentTime);
            const isUpcoming = entry.airing_at > currentTime;

            return (
              <div
                key={`${entry.id}-${entry.episode}`}
                onClick={() => navigate(`/detail/${entry.media_id}`)}
                className="group flex gap-3 p-3 rounded-xl bg-bg-card border border-border hover:border-border-focus hover:bg-bg-elevated cursor-pointer transition-all duration-200 shadow select-none card-hover-glow"
              >
                {/* Poster */}
                <div className="relative w-16 aspect-[2/3] rounded-lg overflow-hidden bg-bg-elevated flex-shrink-0 border border-border">
                  <ShimmerImage
                    src={entry.cover_image}
                    alt={entry.media_title}
                    className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                  />
                </div>

                {/* Details */}
                <div className="flex flex-col justify-between flex-1 min-w-0">
                  <div>
                    <div className="flex items-center justify-between gap-1 mb-1.5">
                      <span className="flex items-center gap-1 text-[10px] font-bold text-accent-300 bg-accent-tint px-2 py-0.5 rounded border border-border-focus">
                        <Clock className="w-3 h-3" />
                        <span>{timeStr}</span>
                      </span>

                      <button
                        type="button"
                        onClick={(e) => handleNotificationToggle(entry, e)}
                        aria-label={isNotified ? 'Disable notification' : 'Enable notification'}
                        className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
                          isNotified
                            ? 'text-accent-300 bg-accent-tint border border-accent-500/40'
                            : 'text-text-muted hover:text-white hover:bg-bg-input'
                        }`}
                      >
                        {isNotified ? <Bell className="w-3.5 h-3.5 fill-current" /> : <BellOff className="w-3.5 h-3.5" />}
                      </button>
                    </div>

                    <h4 className="text-xs font-bold text-text-primary group-hover:text-accent-300 line-clamp-2 transition-colors">
                      {entry.media_title}
                    </h4>
                  </div>

                  <div className="mt-2 pt-1 border-t border-white/5 flex items-center justify-between text-[10px]">
                    <span className="font-semibold text-text-secondary">
                      Episode {entry.episode}
                    </span>
                    <span
                      className={`px-1.5 py-0.5 rounded font-medium ${
                        isUpcoming
                          ? 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
                          : 'bg-white/5 text-white/50'
                      }`}
                    >
                      {countdownStr}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default Schedule;
