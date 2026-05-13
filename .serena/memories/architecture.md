# AsioBridge - Architecture

## Purpose
Modern replacement for ASIO Link Pro — a virtual ASIO driver and audio routing application for Windows. It enables routing audio between multiple applications, virtual ASIO drivers, and the system.

## Tech Stack
- **Backend**: Rust (Cargo workspace, edition 2021)
- **Frontend**: React 19 + TypeScript + Vite 6
- **Desktop Framework**: Tauri 2 (Rust-based, WebKit2GTK frontend)
- **UI Components**: Radix UI primitives
- **Styling**: Tailwind CSS
- **Package Manager**: pnpm
- **Audio**: cpal (Rust audio I/O), rubato (resampling)
- **Build**: CMake (driver), cc + bindgen (FFI)

## Project Structure

```
asiobridge/
├── Cargo.toml              # Workspace root
├── docker-compose.yml      # Linux dev environment
├── crates/
│   ├── asiobridge-core/    # Core audio engine (mixer, resampler, network, profiles, routing)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs
│   │       ├── mixer.rs
│   │       ├── resampler.rs
│   │       ├── network.rs
│   │       ├── profile.rs
│   │       ├── rack.rs
│   │       ├── connection.rs
│   │       ├── audio_stream.rs
│   │       └── recorder.rs
│   ├── asiobridge-driver-sys/  # Driver system bindings (FFI for ASIO/WDM drivers)
│   │   ├── build.rs
│   │   └── src/
│   └── asiobridge-vst-sys/     # VST3 plugin system bindings
├── app/
│   ├── package.json        # Frontend deps (React, Tauri, Radix UI, Tailwind)
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── index.css
│   │   ├── components/     # React UI components
│   │   ├── hooks/          # Custom React hooks
│   │   └── types/          # TypeScript type definitions
│   ├── dist/               # Built frontend assets
│   └── src-tauri/          # Tauri backend
│       ├── tauri.conf.json
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/
├── driver/
│   ├── asiovadpro/         # Virtual audio driver (C, WDK-based)
│   │   ├── asiovadpro.c
│   │   ├── asiovadpro.inf
│   │   ├── asiobridge_driver.h
│   │   └── Makefile
│   └── asiolink/           # ASIO Link compatibility layer (C++)
│       ├── asiobridge_asio.cpp
│       └── CMakeLists.txt
├── installer/              # Installation packaging
├── vendor/                 # Vendored dependencies
└── .github/workflows/
    └── build.yml           # CI/CD pipeline
```

## Key Architecture Layers

1. **AsioBridge App (Tauri 2 + React)** — UI layer with state management
2. **Audio Engine (Rust — asiobridge-core)** — Core audio processing: mixing, resampling, routing, network streaming, recording
3. **ASIO Driver (C++ COM)** — Virtual ASIO driver interface
4. **Virtual Audio Driver (WDK)** — Kernel-level Windows audio driver

## Communication
- Tauri IPC (Rust backend ↔ React frontend)
- UDP network streaming between AsioBridge instances
- Ring buffers (ringbuf crate) for real-time audio data transfer
- Crossbeam channels for inter-thread communication

## Key Dependencies
- **cpal 0.15** — Cross-platform audio I/O
- **rubato 0.14** — Sample rate conversion
- **tokio** — Async runtime
- **serde** — Serialization (profiles)
- **tracing** — Structured logging
