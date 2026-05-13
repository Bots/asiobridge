# AsioBridge - Suggested Commands

## Building

```bash
# Full workspace build
cargo build

# Build Tauri app (frontend + backend)
cd app && pnpm install && pnpm tauri dev

# Build only Rust crates
cargo build --workspace

# Release build
cargo build --release

# Build Tauri for production
cd app && pnpm tauri build
```

## Development

```bash
# Start frontend dev server (Vite)
cd app && pnpm dev

# Start Tauri dev mode (frontend + Rust backend)
cd app && pnpm tauri:dev

# Preview production build
cd app && pnpm preview
```

## Testing

```bash
# Run Rust tests
cargo test

# Run Rust tests for a specific crate
cargo test -p asiobridge-core

# Run TypeScript checks
cd app && npx tsc --noEmit

# Build frontend
cd app && pnpm build
```

## Linting

```bash
# TypeScript linting (via tsc --noEmit)
cd app && npx tsc --noEmit

# Rust formatting
cargo fmt --check

# Rust clippy
cargo clippy --workspace

# Format
cargo fmt
```

## Docker Dev Environment

```bash
# Start dev container
docker-compose up -d
docker exec -it asiobridge-dev bash

# Inside container:
cargo build
cd app && pnpm install
```

## System Dependencies (Ubuntu/Debian)

```bash
sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev \
  mingw-w64 g++-mingw-w64-x86-64
```

## Windows VM Requirements (Driver Development)

- Windows 10/11 with Visual Studio Build Tools + WDK
- Test signing: `bcdedit /set testsigning on`
- Cross-compile toolchain: `x86_64-w64-mingw32-gcc`

## Utility Commands

```bash
# Check workspace dependencies
cargo tree

# Clean build artifacts
cargo clean

# View recent commits
git log --oneline -10
```
