# AsioBridge

Modern replacement for ASIO Link Pro — virtual ASIO driver and audio routing app for Windows.

## Features

- Virtual ASIO driver (64 IN/OUT channels)
- WDM audio mixing (listen to Windows audio while running DAW)
- Multi-client mode (share ASIO driver across 25+ apps)
- Network audio streaming (UDP between AsioBridge instances)
- VST3 plugin hosting and routing
- 8-channel FLAC recording
- Mapping matrix routing with per-channel controls
- 8 profile slots for saving/restoring configurations

## Architecture

```
┌─────────────────────────────────────┐
│  AsioBridge App (Tauri 2 + React)  │
└──────────────┬──────────────────────┘
               │ Tauri IPC (Rust)
┌──────────────┴──────────────────────┐
│  Audio Engine (Rust)                │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│  ASIO Driver (C++ COM)              │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│  Virtual Audio Driver (WDK)         │
└─────────────────────────────────────┘
```

## Development

### Prerequisites

- Rust 1.75+
- pnpm 9+
- Node.js 20+
- CMake 3.20+
- Windows VM with WDK (for driver development)
- Cross-compile toolchain: `x86_64-w64-mingw32-gcc`

### Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install pnpm
corepack enable

# Install system deps (Ubuntu/Debian)
sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  mingw-w64 g++-mingw-w64-x86-64

# Install dependencies
cd app && pnpm install

# Build
cargo build
```

### Windows VM Setup

1. Install Windows 10/11 in VirtualBox/KVM
2. Install Visual Studio Build Tools + WDK
3. Enable network debugging for driver deployment
4. Enable test signing: `bcdedit /set testsigning on`

## License

MIT
