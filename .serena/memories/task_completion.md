# AsioBridge - Task Completion Checklist

## Before Marking a Task Complete

### Rust Changes
1. Run `cargo fmt` to format code
2. Run `cargo clippy --workspace` to check for lint issues
3. Run `cargo test` to ensure all tests pass
4. Run `cargo build --workspace` to ensure it compiles
5. If changes affect audio engine: consider real-time safety implications

### Frontend Changes
1. Run `cd app && pnpm build` to verify TypeScript compiles
2. Run `cd app && pnpm tauri:dev` to test in Tauri dev mode
3. Check for any TypeScript errors or warnings

### Both
1. Verify no breaking changes to public APIs (unless intended)
2. Check Tauri IPC compatibility if backend changes
3. Run `cargo test -p asiobridge-core` for core crate changes
4. Run `cargo test -p asiobridge-app` for Tauri app changes

### Driver Changes (Windows VM Required)
1. Build C driver: `cd driver/asiovadpro && make`
2. Build C++ driver: `cd driver/asiolink && cmake --build .`
3. Test in Windows VM with test signing enabled

## General
- All workspace dependencies should be updated in root `Cargo.toml`
- Frontend dependencies in `app/package.json`
- Version bumps follow workspace convention (currently 0.1.0)
- Commit messages should reference relevant issues
- CI: `.github/workflows/build.yml` handles automated builds
