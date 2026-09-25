<div align="center">

<img src="public/logo.png" alt="Anilili Logo" width="120" style="border-radius: 24px; box-shadow: 0 8px 24px rgba(137, 121, 242, 0.35);" />

# 🎬 Anilili Linux Desktop

### *Native, Ultra-Fast Anime Streaming & Library Management for Linux*

[![CI](https://github.com/mehulgolecha/Anilili_Linux/actions/workflows/ci.yml/badge.svg)](https://github.com/mehulgolecha/Anilili_Linux/actions)
[![Release](https://img.shields.io/badge/Release-v1.0.1-blue.svg)](https://github.com/mehulgolecha/Anilili_Linux/releases)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-FFC131?logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react&logoColor=black)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Linux Platform](https://img.shields.io/badge/Platform-Linux%20(AppImage%20%7C%20RPM%20%7C%20DEB)-E95420?logo=linux&logoColor=white)](https://github.com/mehulgolecha/Anilili_Linux/releases)

<p align="center">
  <b>Anilili Linux</b> is a lightweight, modern, native Linux anime streaming desktop application engineered with <b>Tauri v2</b>, <b>Pure Rust</b>, <b>React 18</b>, and <b>Tailwind CSS</b>. Designed for buttery-smooth 60fps desktop navigation, privacy, and zero telemetry.
</p>

</div>

> [!NOTE]
> **Disclaimer:** Anilili_Linux describes itself as a personal and educational project that is not affiliated with AniList, MyAnimeList, or streaming providers. It hosts no video content, and availability and legality can vary by region. Visitors remain responsible for the laws and terms that apply to them.

---

## 📑 Table of Contents

- [✨ Features](#-features)
  - [Cross-Anime Custom Playlists & Queues](#-cross-anime-custom-playlists--queues)
  - [Next-Gen In-App Player](#-next-gen-in-app-player)
  - [High-Throughput Provider Aggregator](#-high-throughput-provider-aggregator)
  - [Background Download Engine](#-background-download-engine)
  - [External Player Bridge (MPV / VLC)](#-external-player-bridge-mpv--vlc)
  - [AniList & MyAnimeList Synchronization](#-anilist--myanimelist-synchronization)
  - [Local Anime Storage Scanner](#-local-anime-storage-scanner)
- [📦 Installation & Packages](#-installation--packages)
  - [AppImage (Universal Linux)](#appimage-universal-linux)
  - [RPM (Fedora, RHEL, openSUSE)](#rpm-fedora-rhel-opensuse)
  - [DEB (Debian, Ubuntu, Linux Mint)](#deb-debian-ubuntu-linux-mint)
- [⌨️ Keyboard Shortcuts](#-keyboard-shortcuts)
- [🏗️ Architecture & Technology Stack](#-architecture--technology-stack)
- [🛠️ Building From Source](#-building-from-source)
- [⚙️ CI/CD & Automated Packaging](#-cicd--automated-packaging)
- [📄 Project Structure](#-project-structure)
- [⚖️ License](#-license)

---

## ✨ Features

### 📋 Cross-Anime Custom Playlists & Queues
- **Cross-Series Playlists**: Select individual episodes or multi-select entire arcs across different anime titles into custom playlists (e.g. *"Epic Tournament Fights"*, *"Shonen Weekend Marathon"*, *"Slice of Life Comfort"*).
- **Seamless Continuous Playback**: In-app player automatically advances from one anime title to the next, fetching stream sources dynamically.
- **Batch Downloading**: One-click download of all episodes in a playlist directly to your local library via the built-in `aria2c` engine.
- **External Playlist Streaming**: Generate on-the-fly `#EXTM3U8` playlists and stream directly into external media players (**MPV** or **VLC**) with anti-hotlink referer injection.

### 🎥 Next-Gen In-App Player
- **HLS Native Streaming**: Powered by `hls.js` with adaptive bitrate streaming (Auto, 1080p, 720p, 480p, 360p).
- **AniSkip Auto-Skip**: Detects and automatically skips (or offers a one-click floating pill for) opening (OP) and ending (ED) sequences.
- **Customizable Subtitles**: Real-time adjustment of font size, text color, background opacity, and subtitle sync delay (`±ms`).
- **Audio & Subtitle Track Switching**: Effortlessly toggle between SUB (Japanese) and DUB (English) audio and multi-language soft subtitles.
- **Rich Feedback Overlays**: Visual double-tap / arrow key seek indicators (`±5s`, `±10s`), volume scrubbers, and responsive drawer navigation.

### 🌐 High-Throughput Provider Aggregator
- **Zero Python Dependencies**: Direct, asynchronous scraping and stream extraction written in pure Rust with `tokio`, `reqwest`, and `scraper`.
- **25+ Aggregated Stream Sources**: Including Senshi, KickAssAnime, AniBD, AnimeKai, AniDBApp, AnimeGG, AnimeShqip, RareAnimes, Anikoto, AniZone, Miruro, AnimePahe, AnimeHeaven, GojoWtf, and more.
- **Automatic Fallback & Failover**: If a stream provider experiences downtime or anti-bot challenges, Anilili automatically cycles through secondary mirrors.

### ⚡ Background Download Engine
- **Powered by aria2c**: Fast, multi-segmented concurrent connections for blazing download speeds.
- **Format Standardization**: Remuxes HLS `.m3u8` streams and audio tracks into standard `.mp4` files with embedded metadata.
- **Pre-Flight Storage Verification**: Verifies disk space before initiating large series downloads.

### 🚀 External Player Bridge (MPV / VLC)
- **Native MPV & VLC Integration**: Prefer your system's standalone video player? Stream any episode or entire playlist with a single click.
- **Header Forwarding**: Passes required HTTP `Referer` and `Origin` headers to external players to prevent 403 Forbidden hotlink blocks.

### 🔄 AniList & MyAnimeList Synchronization
- **OAuth2 Two-Way Sync**: Authorize your AniList or MyAnimeList account to sync watching status, ratings, and episode progress in real-time.
- **Airing Calendar**: Weekly schedule of broadcast releases with countdown timers and system notification alerts for new episodes.

### 📁 Local Anime Storage Scanner
- **Offline Indexer**: Scans local directories (e.g. `~/Videos/Anime`) and automatically parses filenames, series titles, and episode numbers.
- **Direct Playback**: Watch your offline media directly inside Anilili or dispatch to MPV.

---

## 📦 Installation & Packages

Pre-compiled production packages are available on the [Releases](https://github.com/mehulgolecha/Anilili_Linux/releases) page for every version.

### AppImage (Universal Linux)
Works out of the box on all modern Linux distributions:
```bash
# Make executable
chmod +x Anilili_1.0.1_amd64.AppImage

# Launch Anilili
./Anilili_1.0.1_amd64.AppImage
```

### RPM (Fedora, RHEL, openSUSE)
Native RPM package registered with your desktop environment and system menu:
```bash
# Install via dnf
sudo dnf install ./Anilili-1.0.1-1.x86_64.rpm

# Or install via rpm
sudo rpm -i Anilili-1.0.1-1.x86_64.rpm
```

### DEB (Debian, Ubuntu, Linux Mint)
Native Debian package:
```bash
sudo apt install ./anilili_1.0.1_amd64.deb
```

---

## ⌨️ Keyboard Shortcuts

During in-app video playback:

| Key | Action |
| :--- | :--- |
| <kbd>Space</kbd> / <kbd>K</kbd> | Play / Pause |
| <kbd>F</kbd> | Toggle Fullscreen |
| <kbd>M</kbd> | Toggle Mute |
| <kbd>→</kbd> | Seek Forward 5 seconds |
| <kbd>←</kbd> | Seek Backward 5 seconds |
| <kbd>L</kbd> | Seek Forward 10 seconds |
| <kbd>J</kbd> | Seek Backward 10 seconds |
| <kbd>↑</kbd> | Volume Up (+10%) |
| <kbd>↓</kbd> | Volume Down (-10%) |
| <kbd>Shift</kbd> + <kbd>N</kbd> | Next Episode / Advance Playlist |
| <kbd>Shift</kbd> + <kbd>P</kbd> | Previous Episode |
| <kbd>S</kbd> | Skip Intro (OP) / Outro (ED) |
| <kbd>Esc</kbd> | Exit Fullscreen / Close Drawers |

---

## 🏗️ Architecture & Technology Stack

```
┌────────────────────────────────────────────────────────┐
│                   React 18 Frontend                    │
│   TypeScript • Tailwind CSS • TanStack Query • Zustand │
├────────────────────────────────────────────────────────┤
│                   Tauri v2 IPC Core                    │
│           WebKitGTK Webview (Hardware DMA-BUF)         │
├────────────────────────────────────────────────────────┤
│                   Rust Backend Core                    │
│   tokio • reqwest • sqlx (SQLite) • scraper • aria2c   │
├──────────────────┬──────────────────┬──────────────────┤
│ Stream Providers │ Cloud Providers  │ External Players │
│  Senshi, Pahe,   │  AniList, MAL,   │    MPV, VLC,     │
│  AnimeKai, etc.  │    AnimeThemes   │      aria2c      │
└──────────────────┴──────────────────┴──────────────────┘
```

- **Frontend**: React 18, TypeScript, Tailwind CSS, Vite, TanStack Query v5, Zustand, Lucide React.
- **Backend**: Rust 2021 edition, Tauri v2, Tokio asynchronous runtime, SQLx SQLite, Reqwest with HTTP/2 and TLS, Scraper, Quick-XML.
- **Desktop Packaging**: Native `.AppImage`, `.rpm`, and `.deb` bundles generated via `@tauri-apps/cli`.

---

## 🛠️ Building From Source

### Prerequisites

#### 1. System Dependencies

**Fedora / RHEL:**
```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  aria2 \
  mpv
```

**Ubuntu / Debian:**
```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  aria2 \
  mpv
```

#### 2. Node.js & Rust Toolchains
- [Node.js 18+](https://nodejs.org) (`node -v`)
- [Rust & Cargo](https://rustup.rs) (`rustc --version`)

### Compilation Steps

```bash
# 1. Clone the repository
git clone https://github.com/mehulgolecha/Anilili_Linux.git
cd Anilili_Linux

# 2. Install frontend dependencies
npm install

# 3. Launch live development mode with hot-reloading
npm run tauri dev

# 4. Compile optimized release binaries and packaging bundles
npm run build
npx @tauri-apps/cli build
```

Generated packages will be in:
`src-tauri/target/release/bundle/` (`appimage/`, `rpm/`, `deb/`).

---

## ⚙️ CI/CD & Automated Packaging

The repository includes GitHub Actions workflows under `.github/workflows/`:
- **`ci.yml`**: Runs on every pull request and push to `main`. Executes TypeScript validation, Vite production build, Rust `cargo check`, and the complete test suite.
- **`release.yml`**: Triggers on tag pushes (e.g. `v1.0.1`) or manual workflow dispatch. Compiles production packages across `.AppImage`, `.rpm`, and `.deb` and publishes them to GitHub Releases.

---

## 📄 Project Structure

```
Anilili_Linux/
├── .github/
│   └── workflows/
│       ├── ci.yml               # Automated CI build and testing
│       └── release.yml          # Multi-target release packaging (RPM, DEB, AppImage)
├── src/                         # React Frontend Application
│   ├── components/              # Modular UI components (Player, Playlist, EpisodeBrowser)
│   ├── hooks/                   # Custom TanStack Query & IPC hooks
│   ├── screens/                 # Top-level view screens (Home, Detail, Watch, Library)
│   ├── stores/                  # Zustand client stores (PlayerStore, UIStore)
│   ├── theme/                   # Theme tokens and typography
│   ├── types/                   # TypeScript interfaces and schema types
│   ├── App.tsx                  # Root app layout
│   └── router.tsx               # Hash-based desktop router
├── src-tauri/                   # Tauri v2 Rust Backend
│   ├── src/
│   │   ├── commands/            # Tauri IPC command handlers (Playlists, Streams, Sync)
│   │   ├── downloads/           # aria2c background download controller
│   │   ├── models/              # Serde data structs & SQLite entities
│   │   ├── providers/           # 25+ scraper & extractor implementations
│   │   ├── db.rs                # SQLite migrations & queries
│   │   └── lib.rs               # Application lifecycle & command registration
│   ├── tests/                   # Rust backend unit & integration tests
│   └── tauri.conf.json          # Tauri application and packaging manifest
├── tests/                       # Test suite
├── index.html                   # Desktop webview shell host template
├── package.json                 # Node dependencies and scripts
└── vite.config.ts               # Vite configuration with desktop asset path resolution
```

---

## 🤝 Contributing

Contributions are warmly welcome! Whether you are fixing a bug, adding support for a new streaming provider, improving Linux desktop integration, or refining the UI/UX, your help makes Anilili Linux better for everyone.

### How to Contribute
1. **Fork the Repository** on GitHub.
2. **Create a Feature Branch**:
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make Your Changes**:
   - Adhere to the existing TypeScript and Rust coding conventions.
   - Keep pull requests focused on a single change or feature.
4. **Run Verification & Tests**:
   ```bash
   # Validate frontend build & types
   npm run build

   # Run Rust cargo check and tests
   cargo test --manifest-path src-tauri/Cargo.toml
   ```
5. **Commit Your Changes**:
   ```bash
   git commit -m "feat: add amazing feature"
   ```
6. **Push to Your Branch**:
   ```bash
   git push origin feature/amazing-feature
   ```
7. **Open a Pull Request** describing your changes, motivation, and test results.

---

## 🐛 Bug Reports & Feature Requests

Have you encountered an issue or have a great idea to make Anilili Linux even better? We'd love to hear from you!

- **Bug Reports**: If you experience a crash, playback failure, UI glitch, or broken provider, please [open an issue](https://github.com/mehulgolecha/Anilili_Linux/issues) with:
  - Your Linux distribution and desktop environment (e.g. Fedora 40 GNOME, Arch KDE).
  - Package format used (`.AppImage`, `.rpm`, `.deb`, or built from source).
  - Steps to reproduce the problem and relevant terminal/console logs.
- **Feature Requests**: Have ideas for new player controls, tracker integrations, playlist enhancements, or provider sources? Feel free to [submit a feature request](https://github.com/mehulgolecha/Anilili_Linux/issues) detailing the use case and expected experience.

---

## 💖 Acknowledgments & Inspirations

Anilili Linux stands on the shoulders of the vibrant open-source anime and media software community. Special thanks and heartfelt appreciation to:

- **[Anilili Android](https://github.com/kompoti121/anilili)**: The original Android application that started it all, providing the foundational inspiration, UX vision, and feature set that motivated this native Linux desktop port.
- **[Aniyomi](https://github.com/aniyomiorg/aniyomi) & [Dantotsu](https://github.com/rebelonion/Dantotsu)**: For pioneering mobile anime extensions, robust scrapers, and unified tracker integration patterns.
- **[Seanime](https://github.com/5rahim/seanime)**: For proving how elegant, rich, and responsive modern desktop anime media management can be.
- **[LiSA](https://github.com/LiSA-org)**: For creative inspiration in lightweight desktop anime streaming and sleek UI layouts.
- **[Miru](https://github.com/ThaUnknown/miru)**: For pushing the boundaries of modern desktop anime streaming interfaces and multi-platform experiences.
- **[Tauri](https://tauri.app)** & **[Vite](https://vitejs.dev)**: For enabling the creation of ultra-fast, memory-efficient, and secure native desktop applications without the heavy overhead of Chromium.

---

## ⚖️ License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for more details.
