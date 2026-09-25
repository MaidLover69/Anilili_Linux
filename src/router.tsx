import React, { useEffect } from 'react';
import { createHashRouter, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { onOpenUrl, getCurrent } from '@tauri-apps/plugin-deep-link';
import { TitleBar } from './components/TitleBar';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Sidebar } from './components/Sidebar';
import { ToastContainer } from './components/Toast';
import { AuthDialog } from './components/AuthDialog';
import { BulkDownloadDialog } from './components/BulkDownloadDialog';
import { UpdateDialog } from './components/UpdateDialog';
import { CommandPalette } from './components/CommandPalette';
import { HotkeyCheatSheet } from './components/HotkeyCheatSheet';
import { useUIStore } from './stores/uiStore';
import { Home } from './screens/Home';
import { Discover } from './screens/Discover';
import { Detail } from './screens/Detail';
import { Watch } from './screens/Watch';
import { Library } from './screens/Library';
import { Schedule } from './screens/Schedule';
import { Settings } from './screens/Settings';

const AppLayout: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const isWatchScreen = location.pathname.startsWith('/watch/');
  const {
    commandPaletteOpen,
    setCommandPaletteOpen,
    hotkeyCheatSheetOpen,
    setHotkeyCheatSheetOpen,
  } = useUIStore();

  // Deep linking listener for anilili://anime/:id
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const handleUrl = (urlStr: string) => {
      const match = urlStr.match(/anilili:\/\/anime\/(\d+)/i);
      if (match && match[1]) {
        navigate(`/detail/${match[1]}`);
      }
    };

    getCurrent()
      .then((urls) => {
        if (urls) {
          for (const u of urls) {
            handleUrl(u);
          }
        }
      })
      .catch((err) => {
        console.debug('[DeepLink] getCurrent error or not running in Tauri:', err);
      });

    onOpenUrl((urls) => {
      for (const u of urls) {
        handleUrl(u);
      }
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((err) => {
        console.debug('[DeepLink] onOpenUrl error or not running in Tauri:', err);
      });

    return () => {
      if (unlisten) unlisten();
    };
  }, [navigate]);

  // Global hotkeys listener for Ctrl+K, Cmd+K, and ?
  React.useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Check if user is typing in an input or textarea
      const target = e.target as HTMLElement;
      const isInput =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable;

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCommandPaletteOpen(!commandPaletteOpen);
      } else if (e.key === '?' && !isInput) {
        e.preventDefault();
        setHotkeyCheatSheetOpen(!hotkeyCheatSheetOpen);
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  }, [commandPaletteOpen, hotkeyCheatSheetOpen, setCommandPaletteOpen, setHotkeyCheatSheetOpen]);

  return (
    <div className="flex flex-col w-screen h-screen bg-bg-page text-text-primary overflow-hidden select-none">
      <TitleBar />

      <div className={`flex flex-1 w-full h-[calc(100vh-2rem)] mt-8 overflow-hidden ${isWatchScreen ? 'mt-0 h-screen' : ''}`}>
        {/* Hide sidebar when in full player watch mode */}
        {!isWatchScreen && <Sidebar />}

        {/* Main Content Area */}
        <main
          className={`flex-1 h-full overflow-y-auto ${
            isWatchScreen ? 'p-0' : 'p-6 md:p-8'
          }`}
        >
          <ErrorBoundary>
            <Outlet />
          </ErrorBoundary>
        </main>
      </div>

      {/* Persistent Floating Elements */}
      <ToastContainer />
      <AuthDialog />
      <BulkDownloadDialog />
      <UpdateDialog />
      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
      />
      <HotkeyCheatSheet
        isOpen={hotkeyCheatSheetOpen}
        onClose={() => setHotkeyCheatSheetOpen(false)}
      />
    </div>
  );
};

export const router = createHashRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      {
        index: true,
        element: (
          <ErrorBoundary>
            <Home />
          </ErrorBoundary>
        ),
      },
      {
        path: 'discover',
        element: (
          <ErrorBoundary>
            <Discover />
          </ErrorBoundary>
        ),
      },
      {
        path: 'detail/:id',
        element: (
          <ErrorBoundary>
            <Detail />
          </ErrorBoundary>
        ),
      },
      {
        path: 'watch/:id/:episodeNum',
        element: (
          <ErrorBoundary>
            <Watch />
          </ErrorBoundary>
        ),
      },
      {
        path: 'library',
        element: (
          <ErrorBoundary>
            <Library />
          </ErrorBoundary>
        ),
      },
      {
        path: 'schedule',
        element: (
          <ErrorBoundary>
            <Schedule />
          </ErrorBoundary>
        ),
      },
      {
        path: 'settings',
        element: (
          <ErrorBoundary>
            <Settings />
          </ErrorBoundary>
        ),
      },
      {
        path: '*',
        element: (
          <ErrorBoundary>
            <Home />
          </ErrorBoundary>
        ),
      },
    ],
  },
]);
