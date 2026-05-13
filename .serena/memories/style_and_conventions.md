# AsioBridge - Style and Conventions

## Rust

- **Edition**: 2021 (workspace-wide)
- **Resolver**: 2
- **License**: MIT
- **Error Handling**: thiserror crate for custom error types
- **Logging**: tracing crate with tracing-subscriber (env-filter feature)
- **Naming**: snake_case for functions/variables, PascalCase for types/modules
- **Dependencies**: Workspace-level dependency management in root Cargo.toml
- **Build**: cc crate for C compilation, bindgen for FFI bindings
- **Real-time audio**: Uses ringbuf for lock-free ring buffers, crossbeam-channels for IPC

## TypeScript / React

- **Type**: Strict TypeScript (tsconfig with strict mode)
- **JSX**: JSX transform (modern React 19)
- **Module system**: ESM ("type": "module")
- **Styling**: Tailwind CSS with tailwind-merge for class merging
- **Components**: Radix UI primitives for accessible UI components
- **Icons**: lucide-react
- **File naming**: kebab-case for files, PascalCase for components
- **Structure**: Grouped by feature (components/, hooks/, types/)

## Tauri

- **Version**: Tauri 2
- **Security**: CSP configured in tauri.conf.json
- **Plugin**: shell + store plugins
- **Build**: Frontend dist → ../dist, dev URL on localhost:1420

## Code Organization

- Rust workspace with 3 main crates + Tauri app crate
- Clear separation: core audio engine (Rust), driver (C/C++), UI (React)
- Audio processing crates use synchronous real-time patterns
- Tauri IPC bridges frontend requests to Rust backend
- Profiles use serde/bincode for serialization
