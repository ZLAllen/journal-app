# Journal App (MVP)

Phase 1 foundation for a Tauri + Svelte journaling app.

## Prerequisites

- Rust toolchain (`rustc`, `cargo`)
- Node.js and npm
- Linux build deps for Tauri (Debian):
  - `build-essential`
  - `pkg-config`
  - `libgtk-3-dev`
  - `libwebkit2gtk-4.1-dev`
  - `libsqlite3-dev`

## Project Layout

- `src-tauri/`: Rust backend (commands, db, crypto, models)
- `src/`: Svelte frontend skeleton
- `src-tauri/migrations/`: SQLite schema migrations

## Development

Install frontend dependencies:

```bash
npm install
```

Run backend checks and tests:

```bash
cd src-tauri
cargo check
cargo test
```

Run frontend dev server:

```bash
npm run dev
```

Run Tauri app (requires frontend dev server if configured):

```bash
npm run tauri dev
```

## Implemented in Phase 1

- Tauri backend project scaffolding
- SQLite connection with startup migrations
- Argon2id key derivation helpers in `crypto.rs`
- Age-based encrypt/decrypt helpers
- Data models in `models.rs` with serde derives
- CRUD commands for entries and tag assignment helpers
- Unit and integration tests for backend functionality
