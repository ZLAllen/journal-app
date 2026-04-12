# Journal App — MVP Development Plan

**Target platforms:** Linux and Windows desktop  
**Language:** Rust  
**Goal:** A working, installable desktop journal application covering the core user needs with a minimal but solid feature set.

---

## MVP Scope

The MVP focuses on the highest-value user needs that can be delivered with a single developer or small team in a short cycle. Cloud sync, analytics, and media attachments are deferred post-MVP.

| Requirement area       | MVP | Post-MVP |
|------------------------|-----|----------|
| Entry creation & editing | ✅ | — |
| Auto-save              | ✅ | — |
| Chronological timeline | ✅ | — |
| Full-text search       | ✅ | — |
| Tags and filtering     | ✅ | — |
| Mood logging           | ✅ | — |
| Writing streak         | ✅ | — |
| Passphrase lock        | ✅ | — |
| Data export (JSON)     | ✅ | — |
| Photo attachments      | — | ✅ |
| "On this day" memories | — | ✅ |
| Writing prompts        | — | ✅ |
| Reminder notifications | — | ✅ |
| Cloud sync             | — | ✅ |
| Mood trend charts      | — | ✅ |
| PDF export             | — | ✅ |
| Location logging       | — | ✅ |

---

## Technology Stack

### UI framework — Tauri + a web frontend
Tauri wraps a lightweight Rust backend with a webview-based frontend, producing small, native binaries on both Linux and Windows. It avoids the overhead of Electron while keeping UI development straightforward. The frontend can be written in plain HTML/CSS/JS or with a thin framework like Svelte.

**Alternatives considered:**
- `egui` — immediate-mode Rust GUI, simpler stack but limited rich text support
- `iced` — native Rust GUI, maturing but rich text editing is still limited
- Tauri is preferred here because rich text editing (bold, lists, headings) is mature in the browser via libraries like Tiptap or ProseMirror

### Data storage — SQLite via `rusqlite`
A single local SQLite database file stores all entries, tags, and metadata. SQLite is embedded, requires no server, and is well-supported in Rust via the `rusqlite` crate. Full-text search is handled by SQLite's built-in FTS5 extension.

### Encryption — `ring` or `age`
The database file is encrypted at rest. On MVP, a passphrase is used to derive an AES-256-GCM key via Argon2id (using the `argon2` crate). The `age` crate provides a simpler high-level API if preferred.

### Serialisation — `serde` + `serde_json`
All data models derive `Serialize` / `Deserialize` for JSON export and internal IPC with the Tauri frontend.

### Build and packaging
- `cargo` for the Rust build
- Tauri's built-in bundler produces `.deb` / `.AppImage` on Linux and `.msi` / `.exe` on Windows
- CI via GitHub Actions with cross-compilation targets

---

## Architecture

```
┌─────────────────────────────────────────┐
│              Tauri shell                │
│  ┌───────────────┐  ┌────────────────┐  │
│  │  Webview UI   │  │  Rust backend  │  │
│  │  (Svelte /    │◄─►│  (commands,   │  │
│  │   HTML/CSS)   │  │   db, crypto) │  │
│  └───────────────┘  └────────┬───────┘  │
└───────────────────────────────┼─────────┘
                                │
                    ┌───────────▼──────────┐
                    │  SQLite (FTS5)        │
                    │  encrypted at rest    │
                    └──────────────────────┘
```

The Rust backend exposes Tauri commands (similar to IPC handlers) for every action: creating an entry, querying entries, exporting data, and so on. The frontend never touches the database directly.

---

## Project Structure

```
journal/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Tauri app setup
│   │   ├── commands/
│   │   │   ├── entries.rs   # CRUD for journal entries
│   │   │   ├── search.rs    # FTS5 queries
│   │   │   ├── tags.rs      # Tag management
│   │   │   └── export.rs    # JSON export
│   │   ├── db/
│   │   │   ├── mod.rs       # Connection pool, migrations
│   │   │   └── schema.rs    # Table definitions
│   │   ├── crypto.rs        # Passphrase lock, key derivation
│   │   └── models.rs        # Shared data structs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                     # Frontend (Svelte or HTML/JS)
│   ├── App.svelte
│   ├── routes/
│   │   ├── Timeline.svelte
│   │   ├── Editor.svelte
│   │   └── Search.svelte
│   └── lib/
│       └── api.ts           # Tauri invoke wrappers
├── migrations/              # SQL migration files
└── README.md
```

---

## Data Model

```sql
-- entries
CREATE TABLE entries (
    id          TEXT PRIMARY KEY,   -- UUID v4
    created_at  INTEGER NOT NULL,   -- Unix timestamp (ms)
    updated_at  INTEGER NOT NULL,
    body        TEXT NOT NULL,      -- rich text as HTML or Markdown
    mood        INTEGER,            -- 1–5 scale, nullable
    pinned      INTEGER DEFAULT 0,
    deleted_at  INTEGER             -- soft delete
);

-- tags
CREATE TABLE tags (
    id    TEXT PRIMARY KEY,
    name  TEXT NOT NULL UNIQUE
);

-- entry_tags join
CREATE TABLE entry_tags (
    entry_id  TEXT REFERENCES entries(id),
    tag_id    TEXT REFERENCES tags(id),
    PRIMARY KEY (entry_id, tag_id)
);

-- full-text search
CREATE VIRTUAL TABLE entries_fts USING fts5(
    body,
    content='entries',
    content_rowid='rowid'
);
```

---

## Development Phases

### Phase 1 — Foundation (weeks 1–2)

Stand up the project skeleton and core data layer. No UI yet.

- [x] Initialise Tauri project with Svelte frontend
- [x] Set up SQLite connection with `rusqlite` and run migrations on startup
- [x] Implement `crypto.rs`: derive key from passphrase (Argon2id), encrypt/decrypt the database file using SQLCipher or manual AES-256-GCM on export
- [x] Define data models in `models.rs` with `serde` derives
- [x] Write and test CRUD commands: `create_entry`, `update_entry`, `delete_entry`, `get_entries`
- [x] Set up GitHub Actions CI (lint, test, build on Linux; Windows pending)

**Deliverable:** Rust backend with a working database layer and passing unit tests.

---

### Phase 2 — Core editor and timeline (weeks 3–4)

Build the two primary screens a user will spend most time in.

- [ ] Timeline view: list entries in reverse chronological order, show date, first line of body, mood indicator, and tags
- [ ] Entry editor: integrate a rich text editor (Tiptap via npm) inside the webview; support bold, italic, headings, and bullet lists
- [ ] Auto-save: debounce writes to the backend every 5 seconds while the editor is active
- [ ] Backdating: allow the user to change the `created_at` date on any entry
- [ ] Mood selector: 1–5 scale displayed as simple icons in the editor toolbar
- [ ] Basic tag input: add and remove tags on an entry

**Deliverable:** User can open the app, write entries, and see them listed on the timeline.

---

### Phase 3 — Search, filtering, and organisation (weeks 5–6)

Make the growing collection of entries navigable.

- [ ] Full-text search using SQLite FTS5; return results within 300ms
- [ ] Highlight matched terms in search results
- [ ] Filter panel: filter timeline by date range, tag, and mood
- [ ] Pin/favourite toggle on entries; pinned entries appear at the top of the timeline
- [ ] Tag management screen: rename and delete tags

**Deliverable:** User can find any past entry quickly.

---

### Phase 4 — Security and export (week 7)

Lock the app and give users control over their data.

- [ ] Passphrase lock screen shown on startup and after a configurable idle timeout
- [ ] Key derivation with Argon2id; store the salt in a local config file
- [ ] In-app settings screen: change passphrase, set idle lock timeout
- [ ] JSON export: serialise all entries and tags to a single `.json` file via a save dialog
- [ ] Account deletion: wipe the database file and all app data from settings

**Deliverable:** App is locked by default; user can export and delete all their data.

---

### Phase 5 — Habit tracking and polish (week 8)

Surface the writing streak and ship a stable release candidate.

- [ ] Writing streak: compute consecutive days with at least one entry; display in the sidebar
- [ ] Summary stats: total entries, total word count, entries this month
- [ ] Keyboard shortcuts: `Ctrl+N` new entry, `Ctrl+F` search, `Ctrl+S` manual save
- [ ] Empty states and onboarding copy for first-time users
- [ ] App icon, window title, and About screen
- [ ] Package builds: `.deb`, `.AppImage` for Linux; `.msi` for Windows
- [ ] Manual QA pass on both platforms

**Deliverable:** Shippable MVP build with installers for Linux and Windows.

---

## Key Dependencies

| Crate / Package       | Purpose                                  |
|-----------------------|------------------------------------------|
| `tauri`               | Desktop shell and IPC                    |
| `rusqlite`            | SQLite with FTS5                         |
| `serde`, `serde_json` | Serialisation and JSON export            |
| `argon2`              | Key derivation from passphrase           |
| `ring`                | AES-256-GCM encryption primitives        |
| `uuid`                | Entry ID generation                      |
| `chrono`              | Date/time handling                       |
| `thiserror`           | Ergonomic error types                    |
| `tokio`               | Async runtime (Tauri requires it)        |
| Tiptap (npm)          | Rich text editor in the webview          |
| Svelte (npm)          | Frontend UI framework                    |

---

## Out of Scope for MVP

The following are explicitly deferred to avoid scope creep:

- Cloud sync and multi-device support
- Photo and media attachments
- Push / desktop notifications for reminders
- Writing prompts
- Mood trend charts and analytics dashboard
- PDF export
- Location logging
- Biometric authentication (OS support varies; passphrase is sufficient for MVP)
- "On this day" memories feature

---

## Success Criteria

The MVP is considered shippable when:

1. A user can install the app on a clean Linux or Windows machine without additional dependencies
2. The app opens and is usable within 2 seconds on a mid-range machine
3. A user can create, edit, search, tag, and delete entries without data loss
4. The passphrase lock prevents access to entries without the correct credential
5. A full JSON export of all entries is readable and complete
6. The writing streak counter is accurate across sessions
7. All unit and integration tests pass on both target platforms in CI