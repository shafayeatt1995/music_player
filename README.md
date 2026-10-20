# 🎵 Xorin Player

**Xorin Player** is an ultra-lightweight, high-performance, cross-platform audio player written in 100% pure Rust. It delivers the complete audio power of VLC Media Player in a modern, resource-efficient native desktop experience.

![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)
![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-brightgreen.svg)
![Memory Footprint](https://img.shields.io/badge/RAM-~18MB-success.svg)

## 📦 Quick Installation

### 🍏 macOS (via Homebrew)
```bash
brew tap shafayeatt1995/tap
brew install --cask xorin-player
```

### 🐧 Linux (Debian, Ubuntu, Mint, Pop!_OS)
To install using `apt`:
```bash
# Add Xorin Player APT Repository (one-time setup)
curl -fsSL https://raw.githubusercontent.com/shafayeatt1995/music_player/main/packaging/apt/setup.sh | sudo bash

# Install Xorin Player
sudo apt install xorin-player
```

### ⚡ Universal 1-Line Installer (macOS & Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/shafayeatt1995/music_player/main/install.sh | bash
```

---

## ✨ Features

### 🎬 VLC-Grade Universal Video Player
- **Universal Format Support**: Plays virtually every video container and format supported by VLC: **MP4, MKV, AVI, MOV, WebM, FLV, WMV, TS, M2TS, VOB, MPG, MPEG, 3GP, OGV, DIVX, RMVB, ASF**.
- **Hardware-Accelerated Codecs**: H.264, HEVC / H.265 (including 10-bit), VP8, VP9, AV1, MPEG-2/4, VC-1, ProRes, and more via native VideoToolbox on macOS, DXVA2 on Windows, and VA-API on Linux.
- **Embedded 60 FPS Video Surface**: High-performance double-buffered memory rendering pipeline directly inside the GPU-rendered native GUI.
- **Aspect Ratio Controls**: One-click switching between Original aspect ratio, 16:9 widescreen, 4:3 standard, 21:9 ultrawide cinema, and Fill Window.
- **Immersive Fullscreen Mode**: Press `F` or double-click anywhere on the video for edge-to-edge borderless playback with auto-fading overlay controls.
- **Multi-Track Subtitles & External SRT Loader**: Choose embedded subtitle streams or load external `.srt`, `.sub`, `.ass`, and `.vtt` subtitle files.
- **Multi-Track Audio Language Switching**: Easily cycle between multiple audio language channels in movies and multilingual media.
- **High-Res Video Snapshots**: 1-click snapshot button (`📷 Snapshot`) to capture full-resolution video frames directly to your Pictures folder.

### 🎧 Audio Engine & Codecs
- **Universal Format Support**: MP3, FLAC (Lossless), WAV/PCM, Ogg Vorbis, Opus, AAC, Apple Lossless (ALAC), and AIFF.
- **Hardware-Accelerated Audio I/O**: Direct integration via `cpal` (CoreAudio on macOS, WASAPI on Windows, ALSA/PulseAudio on Linux).
- **Sub-100ms Startup & ~18 MB RAM**: No Chromium, WebKit, or Electron bloat. Starts instantaneously and runs with whisper-quiet CPU usage.

### 🎚 VLC-Grade Equalizer & DSP Audio Suite
- **10-Band Graphic Equalizer**: ISO standard center frequencies (31Hz, 62Hz, 125Hz, 250Hz, 500Hz, 1kHz, 2kHz, 4kHz, 8kHz, 16kHz).
- **Interactive Pre-amp Gain**: Adjustable from -20 dB to +20 dB.
- **13 Professional Audio Presets**: Flat, Rock, Pop, Jazz, Classical, Full Bass, Full Treble, Full Bass & Treble, Club/Dance, Vocal/Acoustic, Techno/Electronic, Reggae, Laptop Speakers/Headphones.
- **VLC 200% Volume Boost**: Smooth digital volume boost up to 200% with hyperbolic tangent (`tanh`) soft-saturation to eliminate harsh clipping.
- **Precision Playback Speed**: Variable playback rate from 0.25x to 4.0x with smooth linear sample interpolation and 1-click 1.0x reset.
- **Stereo Panning**: Fine stereo balance adjustment from Left (-1.0) to Right (+1.0).

### 📊 Real-Time 60 FPS Audio Visualizers
- **FFT Spectrum Analyzer**: 48 frequency bars with logarithmic mapping, peak hold caps, and smooth gravity falloff.
- **Oscilloscope Waveform**: Live time-domain audio wave render.
- **Circular Radial Visualizer**: Dynamic radial spectrum pulsating to the beat.

### 📜 Synchronized Lyrics (LRC)
- **Automatic LRC Discovery**: Detects `.lrc` files next to local music files automatically.
- **Real-Time Auto-Scrolling**: Highlights the active line in glowing amber and auto-scrolls smoothly.
- **Click-to-Seek**: Click any lyric line to jump playback directly to that timestamp.

### 📻 Internet Radio & Live Stream Player
- **Curated High-Uptime Stations**: SomaFM (Groove Salad, Drone Zone, Indie Pop Rocks, Secret Agent, DEF CON Radio, Vaporwaves), Lo-Fi Chillhop, and more.
- **Custom URL Streamer**: Paste any HTTP / HTTPS Icecast, Shoutcast, or direct MP3/AAC audio stream URL to play or add to your playlist.

### 📑 Playlist & Library Management
- **M3U / M3U8 Support**: Full import and export compatibility with VLC and standard music managers.
- **Instant Search & Filter**: Real-time substring filter across titles, artists, albums, and file paths.
- **Playback Modes**: Sequential, Loop All, Loop Single Track, and Fisher-Yates Random Shuffle.
- **Drag-and-Drop**: Drag audio files or whole directories directly into the window.

### 🛠 VLC Practice Tools
- **A-B Looping**: Set Point A and Point B to loop a specific section of a track continuously (ideal for musicians, speech transcription, or language learners).
- **Sleep Timer**: 15m, 30m, 45m, 60m, 90m presets with live countdown to automatically stop playback.
- **Media Information Inspector**: In-depth codec details, sample rate, bit depth, channels, bitrate, and ID3 tags.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|:---|:---|
| `Space` | Play / Pause |
| `S` | Stop |
| `Left Arrow` | Seek backward 5 seconds |
| `Right Arrow` | Seek forward 5 seconds |
| `Shift + Left Arrow` | Seek backward 15 seconds |
| `Shift + Right Arrow` | Seek forward 15 seconds |
| `Up Arrow` | Increase volume (+5%) |
| `Down Arrow` | Decrease volume (-5%) |
| `M` | Toggle Mute |
| `N` | Next track |
| `P` | Previous track |
| `R` | Cycle Repeat mode (Off → All → 1) |
| `Z` | Toggle Shuffle |
| `F` | Toggle Video Fullscreen |
| `Esc` | Exit Video Fullscreen |
| `1` – `7` | Switch tabs (Now Playing, Video, Equalizer, Playlist, Radio, Inspector, Tools) |

---

## 🚀 Building & Running Locally

### Prerequisites
Make sure you have Rust and Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Run in Debug Mode
```bash
cargo run
```

### Build Optimized Release Binary
```bash
cargo build --release
```
The optimized executable will be located at:
- macOS / Linux: `target/release/xorin-player`
- Windows: `target/release/xorin-player.exe`

---

## 🌐 Cross-Platform Build Guide

Xorin Player is built with pure Rust audio decoders (`symphonia`) and GPU rendering (`eframe`/`egui`), eliminating external C library compilation issues across platforms.

### 1. macOS (Apple Silicon M1/M2/M3/M4 & Intel x86_64)

```bash
# Add targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Build for Apple Silicon (M-series)
cargo build --release --target aarch64-apple-darwin

# Build for Intel Mac
cargo build --release --target x86_64-apple-darwin

# Create a Universal macOS Binary (runs natively on BOTH Intel and Apple Silicon):
lipo -create -output xorin-player-universal \
  target/aarch64-apple-darwin/release/xorin-player \
  target/x86_64-apple-darwin/release/xorin-player
```

### 2. Windows (Intel/AMD x86_64 & Snapdragon ARM64)

```bash
# Windows Intel & AMD (x86_64)
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

# Windows Snapdragon / ARM64 laptops
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
```

### 3. Linux (Ubuntu, Debian, Fedora, Arch)

On Linux, install audio and X11/Wayland development headers:
```bash
# Ubuntu / Debian
sudo apt-get install -y libasound2-dev libudev-dev pkg-config libx11-dev libxcursor-dev libxi-dev libxrandr-dev libxinerama-dev libwayland-dev libxkbcommon-dev

# Fedora
sudo dnf install -y alsa-lib-devel systemd-devel libX11-devel libXcursor-devel libXi-devel libXrandr-devel libXinerama-devel wayland-devel libxkbcommon-devel

# Arch Linux
sudo pacman -S alsa-lib systemd libx11 libxcursor libxi libxrandr libxinerama wayland libxkbcommon

# Build
cargo build --release --target x86_64-unknown-linux-gnu
```

---

## 🏗 Architecture Overview

```text
src/
├── main.rs                  # Window bootstrap and viewport initialization
├── app.rs                   # Main XorinApp state coordinator, tabs & keybindings
├── audio/
│   ├── mod.rs               # Audio engine exports
│   ├── backend.rs           # CPAL hardware stream & background DSP worker thread
│   ├── decoder.rs           # Symphonia stream & file decoder
│   ├── dsp.rs               # 10-Band Biquad Equalizer, Preamp & Master DSP
│   ├── visualizer.rs        # FFT processor, spectrum analyzer & oscilloscope buffer
│   └── metadata.rs          # ID3/Vorbis/FLAC tags & album artwork extractor
├── playlist/
│   ├── mod.rs               # Queue, shuffle, repeat & M3U import/export
│   └── track.rs             # Track metadata model
├── lyrics/
│   └── mod.rs               # Synchronized LRC and plain text parser
├── ui/
│   ├── mod.rs               # UI components
│   ├── now_playing.rs       # Artwork, track tags, visualizer & synced lyrics
│   ├── equalizer_view.rs    # 10-Band sliders, preamp & preset selector
│   ├── playlist_view.rs     # Searchable track list and file picker
│   ├── radio_view.rs        # Internet radio presets & custom stream URL player
│   ├── player_bar.rs        # Scrub bar, transport controls, speed & volume boost
│   ├── inspector_view.rs    # VLC-grade codec details and file properties
│   ├── ab_loop.rs           # A-B looping and sleep timer
│   └── visualizer_view.rs   # GPU-accelerated 60 FPS spectrum & waveform rendering
└── utils/
    ├── config.rs            # Settings persistence (JSON in OS config dir)
    └── time.rs              # Time formatting utilities
```

---

## 📄 License
Licensed under either of Apache License, Version 2.0 or MIT License at your option.
