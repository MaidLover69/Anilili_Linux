import React from 'react';
import { NavLink } from 'react-router-dom';
import {
  Home,
  Compass,
  Bookmark,
  Calendar,
  Settings,
  Tv,
  ChevronLeft,
  ChevronRight,
  DownloadCloud,
  Search,
  HelpCircle,
} from 'lucide-react';
import { useUIStore } from '../stores/uiStore';

export const Sidebar: React.FC = () => {
  const {
    sidebarExpanded,
    toggleSidebar,
    setCommandPaletteOpen,
    setHotkeyCheatSheetOpen,
  } = useUIStore();

  const navItems = [
    { to: '/', icon: Home, label: 'Home' },
    { to: '/discover', icon: Compass, label: 'Discover' },
    { to: '/library', icon: Bookmark, label: 'Library' },
    { to: '/schedule', icon: Calendar, label: 'Schedule' },
    { to: '/settings', icon: Settings, label: 'Settings' },
  ];

  return (
    <aside
      className={`relative flex flex-col justify-between h-screen bg-bg-deepest border-r border-border transition-all duration-300 ease-in-out select-none z-30 ${
        sidebarExpanded ? 'w-56 min-w-[220px]' : 'w-16 min-w-[64px]'
      }`}
    >
      {/* Brand Header */}
      <div>
        <div className="flex items-center gap-3 h-16 px-4 border-b border-border">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-tr from-accent-700 to-accent-500 text-white shadow-md shadow-accent-900/30 flex-shrink-0">
            <Tv className="w-5 h-5" />
          </div>
          {sidebarExpanded && (
            <div className="flex flex-col overflow-hidden whitespace-nowrap animate-fade-in">
              <span className="text-base font-bold tracking-wide bg-gradient-to-r from-white via-accent-100 to-accent-300 bg-clip-text text-transparent">
                Anilili
              </span>
              <span className="text-[10px] text-text-muted font-medium tracking-wider uppercase">
                Linux Desktop
              </span>
            </div>
          )}
        </div>

        {/* Navigation Links */}
        <nav className="flex flex-col gap-1.5 p-2 mt-3">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200 group relative ${
                  isActive
                    ? 'bg-accent-tint text-accent-300 border border-border-focus font-semibold'
                    : 'text-text-secondary hover:text-white hover:bg-bg-elevated/60'
                }`
              }
            >
              {({ isActive }) => (
                <>
                  <Icon
                    className={`w-5 h-5 flex-shrink-0 transition-transform duration-200 ${
                      isActive
                        ? 'text-accent-500 scale-110'
                        : 'text-text-muted group-hover:text-text-primary'
                    }`}
                  />
                  {sidebarExpanded && (
                    <span className="whitespace-nowrap overflow-hidden text-ellipsis">
                      {label}
                    </span>
                  )}
                  {!sidebarExpanded && (
                    <div className="absolute left-16 px-2.5 py-1 bg-bg-card text-text-primary border border-border text-xs rounded-md shadow-xl opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-150 z-50 whitespace-nowrap">
                      {label}
                    </div>
                  )}
                </>
              )}
            </NavLink>
          ))}
        </nav>
      </div>

      {/* Footer / Toggle Section */}
      <div className="p-2 border-t border-border flex flex-col gap-1.5">
        {/* Search Ctrl+K Button */}
        <button
          onClick={() => setCommandPaletteOpen(true)}
          className="flex items-center gap-3 px-3 py-2 rounded-lg text-text-muted hover:text-white hover:bg-bg-elevated transition-colors text-xs w-full text-left group relative"
          title="Search / Command Palette (Ctrl+K)"
        >
          <Search className="w-4 h-4 flex-shrink-0 text-accent-500" />
          {sidebarExpanded ? (
            <div className="flex items-center justify-between w-full">
              <span>Quick Search</span>
              <kbd className="px-1.5 py-0.5 text-[10px] bg-white/10 rounded text-white/50">Ctrl+K</kbd>
            </div>
          ) : (
            <div className="absolute left-16 px-2.5 py-1 bg-bg-card text-text-primary border border-border text-xs rounded-md shadow-xl opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-150 z-50 whitespace-nowrap">
              Quick Search (Ctrl+K)
            </div>
          )}
        </button>

        {/* Shortcuts Button */}
        <button
          onClick={() => setHotkeyCheatSheetOpen(true)}
          className="flex items-center gap-3 px-3 py-2 rounded-lg text-text-muted hover:text-white hover:bg-bg-elevated transition-colors text-xs w-full text-left group relative"
          title="Keyboard Shortcuts (?)"
        >
          <HelpCircle className="w-4 h-4 flex-shrink-0 text-emerald-400" />
          {sidebarExpanded ? (
            <div className="flex items-center justify-between w-full">
              <span>Shortcuts</span>
              <kbd className="px-1.5 py-0.5 text-[10px] bg-white/10 rounded text-white/50">?</kbd>
            </div>
          ) : (
            <div className="absolute left-16 px-2.5 py-1 bg-bg-card text-text-primary border border-border text-xs rounded-md shadow-xl opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity duration-150 z-50 whitespace-nowrap">
              Shortcuts (?)
            </div>
          )}
        </button>

        <button
          onClick={toggleSidebar}
          aria-label={sidebarExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
          className="flex items-center justify-center w-full py-1.5 rounded-lg text-text-muted hover:text-white hover:bg-bg-elevated transition-colors text-xs mt-1"
        >
          {sidebarExpanded ? (
            <div className="flex items-center gap-2">
              <ChevronLeft className="w-4 h-4" />
              <span>Collapse</span>
            </div>
          ) : (
            <ChevronRight className="w-4 h-4" />
          )}
        </button>
      </div>
    </aside>
  );
};
