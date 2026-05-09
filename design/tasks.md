# Journal App MVP Task List

This task list is based on the current implementation review against `design/mvp-design.md` and `design/requirements.md`.

## Implementation Progress

### Verified Completed Work

- [x] Tauri + Svelte + TypeScript project exists and builds enough for type checking.
- [x] Rust backend module structure exists for commands, database, crypto helpers, and models.
- [x] Basic local SQLite database initialization and migrations are implemented.
- [x] Basic entry CRUD is implemented in Rust.
- [x] Soft delete is implemented for entries.
- [x] Basic title/body/mood/backdating fields are supported.
- [x] Basic tag creation, attach, remove, rename, delete, and listing are implemented in Rust.
- [x] Entry-tag join table and foreign keys are implemented.
- [x] FTS5 table and maintenance triggers exist for the current `entries.body` column.
- [x] Frontend app shell shows a timeline/editor split view.
- [x] Timeline lists entries with date, title, mood, and tags.
- [x] Tiptap editor is integrated.
- [x] Editor supports bold, italic, heading, and bullet list controls.
- [x] Editor supports mood selection, date editing, and tag add/remove.
- [x] Partial 5-second autosave behavior and save-status UI are implemented.
- [x] Backend sanitizes persisted editor HTML, stores derived plain text, and computes word count.
- [x] FTS5 now indexes the derived plain-text entry projection instead of raw editor HTML.
- [x] Baseline verification passed: `cargo test`.
- [x] Baseline verification passed: `npm run check`.

### Important Implementation Drift

- [ ] Current storage is plain SQLite through `rusqlite`; the MVP requires SQLCipher-compatible encrypted SQLite.
- [ ] Current crypto helpers use Argon2 and `age` for generic payload encryption, but they are not wired into database encryption or app lock.
- [ ] Current entry model uses `title` and `body`; the MVP requires sanitized `body_html`, derived `body_text`, and `word_count`.
- [x] Current Tauri commands return `Result<T, String>` errors; the MVP requires structured `AppError { code, message, recoverable }`.
- [x] Current FTS indexes raw `body`; the MVP requires indexing derived plain text without raw HTML tags.
- [ ] Current app opens directly to journal content; the MVP requires first-run passphrase setup and locked startup after setup.

## Gap Analysis

### Security and Privacy

- [ ] Implement SQLCipher-compatible encrypted SQLite storage.
- [ ] Derive the database key from passphrase using Argon2id.
- [ ] Store salt and KDF parameters in local config separate from the database.
- [ ] Add first-run passphrase setup.
- [ ] Add unlock flow with generic failure messaging and failed-attempt delay.
- [ ] Add lock command that closes the database connection and clears visible journal content.
- [ ] Add change-passphrase flow that rekeys the encrypted database.
- [ ] Add idle lock timeout settings and activity reset behavior.
- [ ] Ensure journal data commands return `LOCKED` before unlock.
- [ ] Confirm logs never include entry content, search query text, tag names, passphrases, or export content.

### Data Model and Command Contract

- [ ] Replace current schema with MVP fields: `body_html`, `body_text`, `word_count`, `created_at_ms`, `updated_at_ms`, and `deleted_at_ms`.
- [ ] Decide whether to remove `title` as implementation drift or revise the MVP spec to include it.
- [ ] Add `app_meta` schema-version storage.
- [ ] Add tag timestamp fields: `created_at_ms` and `updated_at_ms`.
- [ ] Add required indexes for timeline, created date, mood, and entry-tag lookup.
- [x] Implement structured app error serialization with stable error codes.
- [ ] Align command names and payloads with the MVP command specification.
- [x] Add request validation for mood range, entry length, tag names, limits, offsets, and delete confirmation.

### Entries, Editor, Timeline, and Autosave

- [x] Sanitize editor HTML before persistence.
- [x] Allow only MVP-safe tags: `p`, `strong`, `em`, `h1`, `h2`, `h3`, `ul`, `ol`, `li`, and `br`.
- [x] Strip scripts, event handlers, external links, images, styles, iframes, and unknown tags.
- [x] Generate `body_text` from sanitized HTML for previews, search, snippets, and word count.
- [x] Compute and persist `word_count`.
- [x] Add pin toggle in editor and timeline.
- [x] Ensure pinned entries appear first, then all other entries by reverse chronological order.
- [x] Add entry delete UI and confirmation behavior.
- [x] Add empty timeline state with primary new-entry action.
- [x] Add `Ctrl+N` for new entry.
- [x] Add `Ctrl+S` for manual save.
- [ ] Flush pending saves on editor navigation, app close, and lock.
- [x] On app close save failure, show blocking confirmation before closing.
- [x] Preserve dirty editor state after failed autosave and retry every 5 seconds.
- [x] Verify continuous typing still saves periodically, not only after debounce idle.

### Search, Filtering, and Tags

- [x] Add backend `search_entries` command.
- [x] Implement FTS5 query over `entries.body_text`.
- [x] Generate highlighted snippets through FTS5 `snippet()` or `highlight()`.
- [x] Support empty query with filters as filtered timeline results.
- [x] Add date range filter.
- [x] Add tag filter.
- [x] Add mood filter.
- [x] Exclude soft-deleted entries from all search and filter results.
- [x] Replace placeholder `Search.svelte` with real search UI.
- [x] Add no-results empty state.
- [x] Add `Ctrl+F` to focus search.
- [x] Add tag management screen.
- [x] Show tag entry counts.
- [x] Add rename tag UI.
- [x] Add delete tag UI with confirmation.
- [x] Enforce trimmed, non-empty, case-insensitive unique tag names.

### Stats, Settings, Export, and Data Deletion

- [ ] Add `get_summary_stats` command.
- [ ] Implement writing streak using local calendar dates.
- [ ] Implement total entries count.
- [ ] Implement total word count.
- [ ] Implement entries-this-month count.
- [ ] Implement top 10 tags.
- [ ] Add settings screen.
- [ ] Add privacy/about copy matching MVP requirements.
- [ ] Add JSON export command using `journal_mvp_export_v1`.
- [ ] Include entries, tags, mood, timestamps, pinned state, word count, and schema metadata in export.
- [ ] Warn that exported JSON is not encrypted by the app.
- [ ] Write export directly to the user-selected path without persistent plaintext temp files.
- [ ] Add `delete_all_data` command requiring exact `DELETE` confirmation.
- [ ] Delete encrypted database, app config, salt/key metadata, and local caches created by the app.
- [ ] Return app to first-run setup after delete-all-data.

### Accessibility, Performance, CI, and Release

- [x] Add visible focus indicators for all interactive controls.
- [ ] Add accessible names for toolbar, mood, tag, search, filter, settings, export, and delete controls.
- [x] Ensure rich text editor has a meaningful label and does not trap keyboard focus.
- [ ] Verify text contrast meets WCAG 2.1 AA intent.
- [ ] Verify core flows work at 150% text scaling.
- [ ] Add 10,000-entry fixture generator.
- [ ] Add search performance benchmark with 300ms target.
- [ ] Add timeline first-page performance benchmark with 300ms target.
- [ ] Add 50,000-character editor performance scenario.
- [ ] Add export performance scenario for 10,000 entries.
- [ ] Configure CI for Rust format, clippy, unit tests, and integration tests.
- [ ] Configure CI for frontend type check, lint, build, and Tauri build.
- [ ] Configure Linux `.deb` and `.AppImage` builds.
- [ ] Configure Windows `.msi` and `.exe` builds.
- [ ] Complete manual QA on Linux and Windows clean machines.

## Required Public APIs and Interfaces

### Error Contract

- [x] Replace frontend-facing string errors with:

```ts
type AppError = {
  code: string;
  message: string;
  recoverable: boolean;
};
```

### Security Commands

- [ ] `security_has_passphrase()` returns `{ configured: boolean }`.
- [ ] `security_setup_passphrase(passphrase)` returns `{ ok: true }`.
- [ ] `security_unlock(passphrase)` returns `{ ok: true }`.
- [ ] `security_lock()` returns `{ ok: true }`.
- [ ] `security_change_passphrase(old, next)` returns `{ ok: true }`.
- [ ] `security_get_lock_settings()` returns current lock settings.
- [ ] `security_update_lock_settings(settings)` validates and returns updated lock settings.

### Entry Commands

- [ ] `create_entry({ body_html?, mood?, created_at_ms? })` returns `Entry`.
- [ ] `update_entry({ id, body_html?, mood?, created_at_ms?, pinned?, tag_names? })` returns `Entry`.
- [x] `delete_entry({ id })` returns `{ ok: true }`.
- [x] `get_entry({ id })` returns `Entry`.
- [ ] `list_entries({ cursor?, limit?, filters? })` returns `{ entries: EntrySummary[], next_cursor? }`.
- [x] `set_entry_pinned({ id, pinned })` returns `Entry`.

### Search, Tag, Stats, Export, and Data Commands

- [x] `search_entries({ query, filters?, limit?, offset? })` returns `{ results: SearchResult[], elapsed_ms: number }`.
- [x] `list_tags()` returns `Tag[]`.
- [x] `create_tag({ name })` returns `Tag`.
- [x] `rename_tag({ id, name })` returns `Tag`.
- [x] `delete_tag({ id })` returns `{ ok: true }`.
- [ ] `get_summary_stats()` returns `SummaryStats`.
- [ ] `export_json({ path })` returns `{ ok: true, path: string }`.
- [ ] `delete_all_data({ confirmation })` returns `{ ok: true }`.

### Frontend Types

- [ ] Update frontend entry types from current `title/body` shape to MVP `body_html/body_text/word_count` shape.
- [ ] Keep `title` only if the design spec is explicitly revised to include entry titles.
- [ ] Add typed request and response wrappers for each Tauri command.
- [ ] Map structured backend errors to user-facing UI states.

## Phased Development Plan

### Phase 1: Encrypted Foundation and Command Contract

- [x] Initialize Tauri + Svelte + TypeScript project.
- [x] Add initial Rust module structure.
- [x] Implement basic migrations.
- [x] Add initial Rust unit and integration tests.
- [ ] Implement app data path handling for Linux and Windows with config separation.
- [ ] Integrate SQLCipher-compatible SQLite encryption.
- [ ] Implement passphrase setup and unlock.
- [ ] Implement lock-aware app state that opens data commands only after unlock.
- [ ] Align schema and migrations with MVP data model.
- [x] Implement structured app error model.
- [ ] Add locked/unlocked command behavior tests.
- [ ] Add KDF/config serialization tests.

Exit criteria:

- [ ] App can create and unlock an encrypted database.
- [ ] App refuses data commands while locked.
- [ ] Migrations run idempotently.
- [ ] Unit tests pass locally and in CI.

### Phase 2: Entry Editor, Timeline, and Autosave

- [x] Implement basic timeline route.
- [x] Implement editor route with Tiptap.
- [x] Implement basic entry CRUD commands.
- [x] Implement mood selector and backdating.
- [x] Implement basic tag input on entry editor.
- [x] Implement partial autosave loop and save status UI.
- [x] Implement sanitized HTML persistence.
- [x] Implement plain-text extraction and word count.
- [ ] Align entry create/update/list/get commands with MVP payloads.
- [x] Add pinned toggle and persistence.
- [x] Add delete entry UI.
- [ ] Implement lifecycle autosave flush on navigation, close, lock, and `Ctrl+S`.
- [x] Add `Ctrl+N`.
- [ ] Ensure formatting persists after restart.
- [ ] Ensure no plaintext entry content appears in logs.

Exit criteria:

- [ ] User can create, edit, auto-save, backdate, tag, mood-log, pin, and delete entries.
- [ ] Formatting persists after restart.
- [ ] Autosave meets the 5-second and lifecycle requirements.
- [ ] No plaintext entry content is logged.

### Phase 3: Search, Filtering, and Tag Management

- [x] Add initial FTS table and triggers for the current schema.
- [x] Rebuild FTS around MVP `body_text`.
- [x] Implement backend search command.
- [x] Implement search UI with highlighted snippets.
- [x] Implement date, tag, and mood filters.
- [x] Implement tag management screen.
- [x] Implement tag rename/delete behavior in the UI.
- [ ] Add 10,000-entry fixture generator.
- [ ] Add search performance benchmark.

Exit criteria:

- [ ] Search returns results in under 300ms against seeded 10,000-entry data.
- [x] Soft-deleted entries do not appear in search or filters.
- [ ] Tag rename/delete updates correctly without deleting entries.

### Phase 4: Security Settings, Export, and Delete Data

- [ ] Implement lock screen.
- [ ] Implement idle timeout.
- [ ] Implement change passphrase.
- [ ] Implement settings screen.
- [ ] Implement JSON export schema.
- [ ] Implement delete-all-data.
- [ ] Add required privacy/about copy.
- [ ] Add export integration tests.
- [ ] Add delete-all-data integration tests.

Exit criteria:

- [ ] App starts locked after setup.
- [ ] Idle lock works.
- [ ] Passphrase can be changed without data loss.
- [ ] Export file is complete and valid JSON.
- [ ] Delete-all-data returns app to first-run state.

### Phase 5: Stats, Accessibility, Packaging, and Release Candidate

- [ ] Implement writing streak.
- [ ] Implement total entries, total word count, entries this month, and top 10 tags.
- [x] Implement `Ctrl+F`.
- [ ] Complete empty states and onboarding copy.
- [ ] Run accessibility checklist.
- [ ] Add app icon, window title, and About screen polish.
- [ ] Create Linux `.deb` build.
- [ ] Create Linux `.AppImage` build.
- [ ] Create Windows `.msi` build.
- [ ] Create Windows `.exe` installer build.
- [ ] Complete manual QA pass on supported platforms.

Exit criteria:

- [ ] Stats are accurate across restarts and local day boundaries.
- [ ] Core flows pass keyboard and accessibility checks.
- [ ] Installers work on clean Linux and Windows machines.
- [ ] Release candidate is ready for stakeholder acceptance testing.

## Test Plan

### Current Baseline

- [x] `cargo test` passed during review.
- [x] `npm run check` passed during review.

### Unit Tests to Add

- [x] HTML sanitization.
- [x] Plain-text extraction.
- [x] Word count calculation.
- [ ] Date grouping and writing streak calculation.
- [ ] Tag normalization.
- [ ] Input validation.
- [ ] Structured error mapping.
- [ ] KDF config serialization.

### Integration Tests to Add

- [ ] Encrypted database setup and unlock.
- [ ] Locked commands return `LOCKED`.
- [ ] Migration idempotency against the MVP schema.
- [ ] Create, update, get, list, pin, backdate, and delete entry.
- [ ] Soft-deleted entries are excluded from list, search, stats, and export.
- [ ] Tag attach, remove, rename, and delete.
- [ ] FTS trigger sync on insert, update, soft delete, and hard delete.
- [ ] Search filtering combinations.
- [ ] JSON export schema.
- [ ] Delete-all-data using a temporary app data directory.

### Performance Tests to Add

- [ ] Seed 10,000 entries.
- [ ] Include mixed body lengths up to 50,000 characters.
- [ ] Include mixed moods and tags.
- [ ] Verify search under 300ms.
- [ ] Verify timeline first page under 300ms after unlock.
- [ ] Verify export of 10,000 entries completes without crash.

### Manual QA Checklist

- [ ] Clean install on Ubuntu LTS or equivalent mainstream Linux distribution.
- [ ] Clean install on Windows 10 or Windows 11.
- [ ] First-run passphrase setup.
- [ ] Unlock after app restart.
- [ ] Create, edit, pin, backdate, mood-log, tag, and delete entry.
- [ ] Autosave during continuous typing.
- [ ] Close app with pending changes.
- [ ] Search and filter.
- [ ] Tag rename and delete.
- [ ] Change passphrase.
- [ ] Idle lock.
- [ ] Export JSON.
- [ ] Delete all data.
- [ ] Keyboard-only create/edit/search/export path.
- [ ] 150% text scaling sanity check.

## Assumptions and Defaults

- The MVP design spec is the source of truth where it intentionally narrows broader requirements.
- The existing `title` field is treated as implementation drift unless the product spec is revised to include titles.
- SQLCipher-compatible database encryption is required before the app can be considered MVP-ready.
- If SQLCipher support proves infeasible, storage work should stop for an explicit encryption redesign rather than shipping plaintext journal content.
- This file is a planning and acceptance-tracking artifact; it does not itself implement application behavior.
