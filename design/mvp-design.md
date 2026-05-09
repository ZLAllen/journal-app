# Journal App — MVP Development-Ready Design Specification

**Target platforms:** Linux and Windows desktop  
**Language:** Rust  
**Frontend:** Svelte + TypeScript running in Tauri WebView  
**Backend:** Rust Tauri commands  
**Storage:** Local SQLite database encrypted at rest  
**MVP release goal:** A working, installable, local-first desktop journal application that supports secure private writing, retrieval, organization, export, and habit tracking.

---

## 1. MVP Scope Contract

The original product requirements describe a broader cross-device journaling product, including iOS, Android, responsive web, cloud sync, reminders, media attachments, mood trend charts, PDF export, and location logging.

This MVP deliberately narrows scope to a **local-first Linux and Windows desktop application**. The MVP is intended to validate the core desktop journaling loop before investing in cloud, mobile, and multi-device infrastructure.

### 1.1 In Scope for MVP

| Capability | MVP decision |
|---|---|
| Desktop app | Linux and Windows only |
| Entry creation and editing | Included |
| Rich text formatting | Included: bold, italic, headings, bullet lists |
| Auto-save | Included: periodic save every 5 seconds plus flush on navigation/close |
| Timeline browsing | Included: reverse chronological timeline |
| Search | Included: full-text search with highlighted matches |
| Filters | Included: date range, tags, mood |
| Tags | Included: create, attach, remove, rename, delete |
| Mood logging | Included: optional 1–5 mood value per entry |
| Pin/favourite | Included |
| Writing streak | Included |
| Summary stats | Included: total entries, total words, entries this month, top 10 tags |
| App lock | Included: passphrase-based lock |
| Data at rest | Included: encrypted local database |
| Export | Included: JSON export |
| Delete all data | Included |
| Accessibility | Included for core flows |
| Installers | Included: Linux `.deb` / `.AppImage`; Windows `.msi` / `.exe` |

### 1.2 Explicitly Out of Scope for MVP

| Capability | Reason deferred |
|---|---|
| iOS and Android apps | Requires separate native/mobile product work |
| Responsive web app | Requires hosted service and web auth model |
| Cloud sync | Requires backend, auth, conflict resolution, uptime commitment |
| TLS 1.3 in transit | No network sync in MVP; not applicable to local-only app |
| 99.9% cloud uptime | No cloud service in MVP |
| Add entries from any device | Requires sync and mobile/web clients |
| Photo/media attachments | Adds file storage, export, and privacy complexity |
| Media presence filter | Not applicable without media attachments |
| Location logging | Sensitive data and platform complexity |
| Reminder notifications | OS notification implementation deferred |
| Writing prompts | Content/product feature deferred |
| “On this day” memories | Deferred retrieval/insight feature |
| Mood trend charts | Deferred analytics visualization |
| PDF export | Deferred export format |
| PIN and biometric unlock | Replaced by passphrase for MVP; may be added later |

### 1.3 MVP Success Criteria

The MVP is shippable when all of the following are true:

1. A user can install and launch the app on a clean supported Linux or Windows machine without extra dependencies.
2. The app opens to the locked or usable home state within **2 seconds** on target hardware.
3. A user can create, edit, backdate, tag, search, filter, pin, and delete journal entries without data loss.
4. Entries up to **50,000 characters** remain editable and searchable without unacceptable lag.
5. Full-text search returns results within **300ms** for a seeded database of **10,000 entries** on target hardware.
6. The app encrypts local journal data at rest and does not expose readable entry content before unlock.
7. Auto-save persists edits within 5 seconds and flushes pending changes on navigation, app close, and manual save.
8. JSON export contains all non-deleted entries, tags, mood values, timestamps, pinned state, and schema metadata.
9. Delete-all-data removes the encrypted database, app config, salt/key metadata, and local caches created by the app.
10. Writing streak and summary stats are accurate across app restarts and local day boundaries.
11. Core flows are usable with keyboard navigation and basic screen reader labels.
12. Unit, integration, and smoke tests pass in CI for Linux and Windows.

---

## 2. Product Requirements Traceability

| Original requirement | MVP handling | Notes |
|---|---|---|
| Open app and start writing quickly | Included | Home/editor usable within 2 seconds after unlock |
| Save automatically | Included | 5-second periodic auto-save plus flush events |
| Add entries from any device | Deferred | Requires cloud/mobile/web |
| Chronological timeline | Included | Reverse chronological, pinned first |
| Full-text search | Included | SQLite FTS5 |
| “On this day” memories | Deferred | Post-MVP |
| Writing streak | Included | Sidebar summary |
| Reminders | Deferred | Post-MVP |
| Prompts | Deferred | Post-MVP |
| Tags | Included | Entry tags and tag management |
| Filters by tag/date/mood | Included | Media filter deferred as no media |
| Pin/favourite entries | Included | Pinned group shown above normal timeline |
| Photo attachments | Deferred | Post-MVP |
| Mood/energy level | Included | Mood 1–5; energy not included separately |
| Location | Deferred | Post-MVP |
| App lock | Included | Passphrase only for MVP |
| Clear storage/privacy explanation | Included | Settings/About privacy copy |
| Export/delete data | Included | JSON export and delete-all-data |
| Mood trend charts | Deferred | Post-MVP |
| Top 10 tags | Included | Summary stat only, no full analytics dashboard |
| AES-256 encryption at rest | Included | SQLCipher-backed encrypted database |
| TLS 1.3 in transit | Not applicable | No network traffic for journal data |
| GDPR/CCPA | Partially addressed | Local-only export/delete; no cloud data processing |
| Native mobile + responsive web | Deferred | Desktop-only MVP |
| WCAG 2.1 AA | Included for core desktop flows | See accessibility acceptance criteria |

---

## 3. Architecture

### 3.1 Architectural Style

The application is a local-first Tauri desktop app. The frontend renders all UI and calls Rust backend commands through Tauri IPC. The frontend never opens or directly manipulates the database.

```text
┌──────────────────────────────────────────────┐
│                 Tauri shell                  │
│                                              │
│  ┌────────────────┐      ┌────────────────┐  │
│  │ Svelte WebView │◄────►│ Rust commands  │  │
│  │ UI + Tiptap    │ IPC  │ services layer │  │
│  └────────────────┘      └───────┬────────┘  │
│                                  │           │
│                          ┌───────▼────────┐  │
│                          │ Repository     │  │
│                          │ + migrations   │  │
│                          └───────┬────────┘  │
│                                  │           │
│                          ┌───────▼────────┐  │
│                          │ SQLCipher DB   │  │
│                          │ SQLite + FTS5  │  │
│                          └────────────────┘  │
└──────────────────────────────────────────────┘
```

### 3.2 Layers

| Layer | Responsibility |
|---|---|
| UI layer | Rendering, editor state, forms, routing, accessibility labels, client-side validation |
| API wrapper layer | Typed Tauri `invoke` wrappers and error mapping |
| Tauri command layer | Stable command interface, request validation, response serialization |
| Service layer | Business rules: autosave, stats, search, tags, lock lifecycle, export |
| Repository layer | SQL queries, migrations, transactions, FTS sync |
| Crypto/lock layer | Passphrase verification, SQLCipher key handling, idle lock state |
| Platform layer | App data paths, file dialogs, packaging, OS-specific behavior |

---

## 4. Technology Decisions

### 4.1 UI Framework

**Decision:** Tauri + Svelte + TypeScript.

Rationale:
- Produces smaller desktop binaries than Electron.
- Keeps Rust backend for database, file, and crypto-sensitive operations.
- Allows use of mature browser-based rich text editing libraries.

### 4.2 Rich Text Editor

**Decision:** Tiptap.

Supported MVP formatting:
- Paragraph
- Bold
- Italic
- Heading level 1–3
- Bullet list
- Ordered list may be implemented if available cheaply, but bullet lists are required

### 4.3 Rich Text Storage Format

**Decision:** Store entry body as sanitized HTML.

Rationale:
- Tiptap can produce HTML reliably.
- HTML preserves MVP formatting without markdown conversion edge cases.
- FTS indexing can use a plain-text projection generated from sanitized HTML.

Rules:
- Persist sanitized HTML in `entries.body_html`.
- Persist plain-text projection in `entries.body_text` for search snippets, previews, word count, and FTS indexing.
- Do not index raw HTML tags.
- Sanitize all editor output before persistence.
- Allowed tags: `p`, `strong`, `em`, `h1`, `h2`, `h3`, `ul`, `ol`, `li`, `br`.
- Strip scripts, inline event handlers, external links, images, styles, iframes, and unknown tags in MVP.

### 4.4 Database

**Decision:** SQLite encrypted using SQLCipher-compatible encryption.

Implementation requirement:
- The local database file must not contain readable journal body text, tags, moods, or timestamps without the passphrase-derived key.
- The app must not create unencrypted shadow copies, temp export files, logs, or crash dumps containing journal bodies.

Preferred implementation:
- Use a Rust SQLite binding/build that supports SQLCipher.
- Apply the database key immediately after opening the connection and before running migrations or queries.

Fallback rule:
- If SQLCipher support proves infeasible, the team must stop and re-design encryption before implementing storage. Do not silently ship unencrypted local journal content.

### 4.5 Search

**Decision:** SQLite FTS5 over `entries.body_text`.

Rules:
- FTS index contains plain text, but because the database file is encrypted, the FTS table is encrypted at rest too.
- FTS rows are kept synchronized via database triggers.
- Deleted entries are excluded from search results.
- Search uses sanitized user input and parameterized queries.

### 4.6 Time Handling

**Decision:** Store timestamps as Unix epoch milliseconds in UTC and compute user-facing local dates using the OS local timezone at runtime.

Rules:
- `created_at_ms`, `updated_at_ms`, and `deleted_at_ms` are UTC epoch milliseconds.
- Streak calculations group entries by local calendar date.
- If the OS timezone changes, streak calculation uses the current OS timezone. This is acceptable for MVP.
- Backdating updates `created_at_ms` only. `updated_at_ms` reflects actual modification time.

---

## 5. Project Structure

```text
journal/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── app_state.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── entries.rs
│   │   │   ├── search.rs
│   │   │   ├── tags.rs
│   │   │   ├── stats.rs
│   │   │   ├── security.rs
│   │   │   ├── settings.rs
│   │   │   └── export.rs
│   │   ├── services/
│   │   │   ├── entries_service.rs
│   │   │   ├── search_service.rs
│   │   │   ├── tags_service.rs
│   │   │   ├── stats_service.rs
│   │   │   ├── lock_service.rs
│   │   │   └── export_service.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs
│   │   │   ├── migrations.rs
│   │   │   └── repositories.rs
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   ├── key_derivation.rs
│   │   │   └── encrypted_db.rs
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── entry.rs
│   │   │   ├── tag.rs
│   │   │   ├── stats.rs
│   │   │   └── errors.rs
│   │   └── platform/
│   │       ├── paths.rs
│   │       └── dialogs.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.svelte
│   ├── routes/
│   │   ├── Unlock.svelte
│   │   ├── Timeline.svelte
│   │   ├── Editor.svelte
│   │   ├── Search.svelte
│   │   ├── Tags.svelte
│   │   └── Settings.svelte
│   ├── components/
│   │   ├── EntryListItem.svelte
│   │   ├── EditorToolbar.svelte
│   │   ├── MoodSelector.svelte
│   │   ├── TagInput.svelte
│   │   ├── FilterPanel.svelte
│   │   └── SaveStatus.svelte
│   ├── lib/
│   │   ├── api.ts
│   │   ├── types.ts
│   │   ├── sanitize.ts
│   │   ├── text.ts
│   │   └── shortcuts.ts
│   └── styles/
│       └── app.css
├── migrations/
│   ├── 0001_initial.sql
│   └── 0002_fts_triggers.sql
├── tests/
│   ├── integration/
│   └── fixtures/
└── README.md
```

---

## 6. Data Model

### 6.1 SQLite Schema

```sql
CREATE TABLE app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE entries (
    id TEXT PRIMARY KEY,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    deleted_at_ms INTEGER,
    body_html TEXT NOT NULL,
    body_text TEXT NOT NULL,
    mood INTEGER CHECK (mood IS NULL OR mood BETWEEN 1 AND 5),
    pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
    word_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);

CREATE TABLE entry_tags (
    entry_id TEXT NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id)
);

CREATE VIRTUAL TABLE entries_fts USING fts5(
    body_text,
    content='entries',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE INDEX idx_entries_timeline
    ON entries(deleted_at_ms, pinned DESC, created_at_ms DESC);

CREATE INDEX idx_entries_created_at
    ON entries(created_at_ms DESC);

CREATE INDEX idx_entries_mood
    ON entries(mood);

CREATE INDEX idx_entry_tags_tag
    ON entry_tags(tag_id, entry_id);

CREATE INDEX idx_entry_tags_entry
    ON entry_tags(entry_id, tag_id);
```

### 6.2 FTS Triggers

```sql
CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, body_text)
    VALUES (new.rowid, new.body_text);
END;

CREATE TRIGGER entries_ad AFTER DELETE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, body_text)
    VALUES ('delete', old.rowid, old.body_text);
END;

CREATE TRIGGER entries_au AFTER UPDATE OF body_text ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, body_text)
    VALUES ('delete', old.rowid, old.body_text);
    INSERT INTO entries_fts(rowid, body_text)
    VALUES (new.rowid, new.body_text);
END;
```

Soft-deleted entries remain in `entries` but must be excluded by all list/search/stat queries using `deleted_at_ms IS NULL`.

### 6.3 Export Schema

JSON export format version: `journal_mvp_export_v1`.

```json
{
  "format": "journal_mvp_export_v1",
  "exported_at_ms": 1760000000000,
  "app_version": "0.1.0",
  "entries": [
    {
      "id": "uuid",
      "created_at_ms": 1760000000000,
      "updated_at_ms": 1760000000000,
      "body_html": "<p>Hello</p>",
      "body_text": "Hello",
      "mood": 4,
      "pinned": false,
      "word_count": 1,
      "tags": ["gratitude", "work"]
    }
  ],
  "tags": [
    {
      "id": "uuid",
      "name": "gratitude",
      "created_at_ms": 1760000000000,
      "updated_at_ms": 1760000000000
    }
  ]
}
```

Rules:
- Export includes non-deleted entries only.
- Export includes all tags currently attached to non-deleted entries and any standalone tags that still exist.
- Export file is user-selected through a save dialog.
- Export is written directly to the selected path; no persistent temp file containing plaintext is left behind.
- Export is plaintext JSON by design because the user requested export. Settings copy must warn that exported files are not encrypted by the app.

---

## 7. Backend Command Specification

All commands return typed success responses or a structured error:

```ts
type AppError = {
  code: string;
  message: string;
  recoverable: boolean;
};
```

### 7.1 Security Commands

| Command | Request | Response | Notes |
|---|---|---|---|
| `security_has_passphrase()` | none | `{ configured: boolean }` | Used on startup |
| `security_setup_passphrase(passphrase)` | passphrase | `{ ok: true }` | Creates encrypted DB/config |
| `security_unlock(passphrase)` | passphrase | `{ ok: true }` | Opens encrypted DB session |
| `security_lock()` | none | `{ ok: true }` | Clears active DB handle/key from memory where practical |
| `security_change_passphrase(old, next)` | old/new passphrase | `{ ok: true }` | Re-keys encrypted DB |
| `security_get_lock_settings()` | none | settings | Includes idle timeout |
| `security_update_lock_settings(settings)` | settings | settings | Validates timeout |

### 7.2 Entry Commands

| Command | Request | Response |
|---|---|---|
| `create_entry` | `{ body_html?, mood?, created_at_ms? }` | `Entry` |
| `update_entry` | `{ id, body_html?, mood?, created_at_ms?, pinned?, tag_names? }` | `Entry` |
| `delete_entry` | `{ id }` | `{ ok: true }` |
| `get_entry` | `{ id }` | `Entry` |
| `list_entries` | `{ cursor?, limit?, filters? }` | `{ entries: EntrySummary[], next_cursor? }` |
| `set_entry_pinned` | `{ id, pinned }` | `Entry` |

Rules:
- `create_entry` creates an entry only after first non-empty content or explicit user action to create a blank entry.
- `update_entry` sanitizes HTML, computes `body_text`, updates `word_count`, updates FTS through triggers, and sets `updated_at_ms`.
- `delete_entry` soft-deletes by setting `deleted_at_ms`.
- Default `list_entries` limit is 50.
- Pinned entries appear first, then remaining entries by `created_at_ms DESC`.

### 7.3 Search Commands

| Command | Request | Response |
|---|---|---|
| `search_entries` | `{ query, filters?, limit?, offset? }` | `{ results: SearchResult[], elapsed_ms: number }` |

Rules:
- Empty query with filters returns filtered timeline results.
- Non-empty query uses FTS5.
- Results exclude soft-deleted entries.
- Highlighted snippets are generated using FTS5 `snippet()` or `highlight()` from `body_text`.
- Default result limit is 50.

### 7.4 Tag Commands

| Command | Request | Response |
|---|---|---|
| `list_tags` | none | `Tag[]` |
| `create_tag` | `{ name }` | `Tag` |
| `rename_tag` | `{ id, name }` | `Tag` |
| `delete_tag` | `{ id }` | `{ ok: true }` |

Rules:
- Tag names are trimmed.
- Empty tag names are rejected.
- Duplicate names are rejected case-insensitively.
- Deleting a tag removes join records but not entries.

### 7.5 Stats Commands

| Command | Request | Response |
|---|---|---|
| `get_summary_stats` | none | `SummaryStats` |

`SummaryStats`:

```ts
type SummaryStats = {
  writing_streak_days: number;
  total_entries: number;
  total_word_count: number;
  entries_this_month: number;
  top_tags: Array<{ name: string; count: number }>;
};
```

Rules:
- Stats exclude soft-deleted entries.
- Writing streak counts consecutive local calendar days ending today if the user has written today, otherwise ending yesterday.
- A day counts if it has at least one non-deleted entry with non-empty `body_text`.

### 7.6 Export and Data Commands

| Command | Request | Response |
|---|---|---|
| `export_json` | `{ path }` | `{ ok: true, path: string }` |
| `delete_all_data` | `{ confirmation: string }` | `{ ok: true }` |

Rules:
- `delete_all_data` requires exact confirmation string: `DELETE`.
- After deletion, app returns to first-run setup state.
- Export fails safely if the selected path is not writable.

---

## 8. Frontend UX Specification

### 8.1 First Run

1. User opens app.
2. If no passphrase is configured, show setup screen.
3. User enters passphrase and confirmation.
4. App creates encrypted database and opens timeline.
5. Show short local-only privacy notice:
   - Journal data is stored locally on this device.
   - The app does not sync data in MVP.
   - Losing the passphrase may make the journal unrecoverable.

Acceptance criteria:
- Passphrase and confirmation must match.
- Minimum passphrase length is 10 characters.
- User can proceed using only keyboard.

### 8.2 Unlock Flow

1. On launch after setup, app shows unlock screen.
2. User enters passphrase.
3. Correct passphrase opens timeline.
4. Incorrect passphrase shows generic error: “Could not unlock journal. Check your passphrase and try again.”

Rules:
- Do not reveal whether passphrase, file, or key was the exact failure cause.
- Add a 1-second delay after failed unlock attempts.
- No account recovery in MVP.

### 8.3 Timeline

Timeline displays:
- Pinned entries first.
- Then non-pinned entries in reverse chronological order.
- Entry date.
- Plain-text preview from `body_text`.
- Mood indicator if present.
- Tags.
- Pinned state.

Actions:
- New entry.
- Open entry.
- Search.
- Filter.
- Settings.
- Tag management.

Empty state:
- “Start your first journal entry.”
- Primary action: “New entry”.

### 8.4 Editor

Editor supports:
- Rich text body.
- Toolbar: bold, italic, heading, bullet list.
- Mood selector: 1–5 or unset.
- Tag input.
- Created date picker for backdating.
- Pin toggle.
- Save status indicator.

Save status values:
- `Saved`
- `Saving…`
- `Unsaved changes`
- `Could not save. Retrying…`

### 8.5 Auto-save

Behavior:
- After the user changes content, mark editor as `Unsaved changes`.
- Save at least once every 5 seconds while there are unsaved changes.
- Also save immediately when:
  - user navigates away from the editor,
  - app window receives close event,
  - app is locked,
  - user presses `Ctrl+S`.
- If a save fails, keep the editor content in memory and retry after 5 seconds.
- If app close occurs while save is pending, attempt final save before close. If save fails, show blocking confirmation: “Your latest changes could not be saved. Close anyway?”

Implementation note:
- Use a throttled autosave loop, not pure debounce-only behavior. Continuous typing must still save periodically.

### 8.6 Search and Filters

Search screen includes:
- Search input.
- Date range filter.
- Tag filter.
- Mood filter.
- Clear filters action.

Search result displays:
- Entry date.
- Highlighted matched snippet.
- Tags.
- Mood.

Acceptance criteria:
- Search query is keyboard-focusable with `Ctrl+F`.
- Highlighting is visible and screen-reader text remains understandable.
- Search with no results shows a helpful empty state.

### 8.7 Tag Management

Tag management screen supports:
- List all tags with entry counts.
- Rename tag.
- Delete tag.

Delete behavior:
- Confirm deletion.
- Removing a tag does not delete entries.

### 8.8 Settings

Settings supports:
- Change passphrase.
- Set idle lock timeout.
- Export JSON.
- Delete all data.
- About/privacy section.

Idle lock timeout options:
- Never during current session
- 1 minute
- 5 minutes default
- 15 minutes
- 30 minutes

### 8.9 Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+N` | New entry |
| `Ctrl+F` | Focus search |
| `Ctrl+S` | Force save current entry |
| `Esc` | Close modal/filter panel where applicable |

---

## 9. Security and Privacy Specification

### 9.1 Encryption

- Use SQLCipher-compatible encrypted SQLite database.
- Derive database key from passphrase using Argon2id.
- Store random salt and KDF parameters in local config separate from the DB.
- Store no plaintext journal body content in config, logs, or unencrypted files.
- Use AES-256 database encryption via SQLCipher-compatible configuration.

KDF baseline:
- Algorithm: Argon2id.
- Salt: 16+ random bytes.
- Parameters must be set high enough to slow offline guessing while keeping unlock acceptable on target hardware.
- Store parameter values to allow future migration.

### 9.2 Lock State

- Before unlock, commands that access journal data return `LOCKED` error.
- On lock, close encrypted DB connection and clear passphrase/key material from app state where practical.
- UI clears visible journal content when locked.
- App locks on startup by default after setup.
- Idle timer resets on keyboard/mouse activity inside the app.

### 9.3 Privacy Copy

The About/Privacy section must state:
- Journal data is stored locally on this device in an encrypted database.
- The MVP does not sync data to a server.
- The app developers cannot recover a lost passphrase.
- Exported JSON files are not encrypted by the app.
- Delete-all-data removes local app data for this device.

### 9.4 Logging

Logs may include:
- App start/stop.
- Error codes.
- Timing metrics.

Logs must not include:
- Entry content.
- Search query text.
- Tag names.
- Passphrases.
- Export file content.

---

## 10. Performance Requirements

| Area | Requirement | Test method |
|---|---|---|
| App launch | Home/unlock screen visible within 2 seconds | Smoke test/manual QA |
| Timeline load | First 50 entries load within 300ms after unlock | Integration benchmark |
| Editor | 50,000-character entry editable without visible freeze over 500ms | Manual/performance test |
| Autosave | Save completes within 500ms for typical entry under 10,000 chars | Integration benchmark |
| Search | FTS results within 300ms for 10,000 seeded entries | Automated benchmark |
| Export | 10,000 entries exported without crash | Integration test |

Target hardware:
- Mid-range laptop or desktop from the last 5 years.
- 8GB RAM.
- SSD storage.

---

## 11. Accessibility Requirements

Core flows must satisfy MVP-level WCAG 2.1 AA intent.

Acceptance criteria:
- All primary flows are keyboard accessible: setup, unlock, create entry, edit entry, save, search, filter, export, delete data.
- Visible focus indicator is present for all interactive controls.
- Buttons, inputs, mood selector, tag input, search filters, and toolbar controls have accessible names.
- Text contrast meets WCAG AA for normal text and UI controls.
- App remains usable at 150% OS/browser text scaling.
- Rich text editor exposes a meaningful label and does not trap keyboard focus.
- Error messages are associated with relevant fields where possible.

---

## 12. Error Handling

### 12.1 Error Codes

| Code | Meaning | User-facing behavior |
|---|---|---|
| `LOCKED` | Journal is locked | Show unlock screen |
| `INVALID_INPUT` | Request failed validation | Show field-level error |
| `NOT_FOUND` | Entry/tag not found | Show non-blocking error and refresh list |
| `DB_ERROR` | Database operation failed | Show retryable error |
| `ENCRYPTION_ERROR` | Unlock/rekey/encrypted DB failure | Show generic security error |
| `EXPORT_ERROR` | Export failed | Show path/writability guidance |
| `PERMISSION_ERROR` | OS denied file access | Ask user to choose another location |
| `UNKNOWN` | Unexpected error | Show generic error and preserve user edits |

### 12.2 Data Loss Prevention

- Editor keeps latest unsaved content in memory until save succeeds or user explicitly discards.
- Failed autosave must not clear the dirty state.
- Navigating away attempts save first.
- Closing with failed save requires explicit user confirmation.

---

## 13. Development Phases

### Phase 1 — Foundation and Encrypted Data Layer

Tasks:
- Initialize Tauri + Svelte + TypeScript project.
- Add Rust module structure.
- Implement app data path handling for Linux and Windows.
- Integrate SQLCipher-compatible SQLite encryption.
- Implement passphrase setup and unlock.
- Implement migrations.
- Implement base schema, indexes, and FTS triggers.
- Implement typed command error model.
- Implement unit tests for KDF/config and locked/unlocked command behavior.
- Set up CI for Linux and Windows: format, lint, unit tests, build.

Exit criteria:
- App can create and unlock encrypted database.
- App refuses data commands while locked.
- Migrations run idempotently.
- Unit tests pass in CI.

### Phase 2 — Entry Editor, Timeline, and Autosave

Tasks:
- Implement timeline route.
- Implement editor route with Tiptap.
- Implement HTML sanitization and plain-text extraction.
- Implement entry CRUD commands.
- Implement mood selector and backdating.
- Implement tag input on entry editor.
- Implement pinned toggle.
- Implement autosave loop and save status UI.
- Implement `Ctrl+N` and `Ctrl+S`.

Exit criteria:
- User can create, edit, auto-save, backdate, tag, mood-log, pin, and delete entries.
- Formatting persists after restart.
- Autosave meets lifecycle requirements.
- No plaintext entry content appears in logs.

### Phase 3 — Search, Filtering, and Tag Management

Tasks:
- Implement FTS search command.
- Implement search UI and highlighted snippets.
- Implement date/tag/mood filters.
- Implement tag management screen.
- Implement tag rename/delete behavior.
- Add 10,000-entry fixture generator.
- Add search performance benchmark.

Exit criteria:
- Search returns results in under 300ms against seeded data.
- Soft-deleted entries do not appear in search or filters.
- Tag rename/delete updates correctly.

### Phase 4 — Security Settings, Export, and Delete Data

Tasks:
- Implement lock screen and idle timeout.
- Implement change passphrase.
- Implement settings screen.
- Implement JSON export schema.
- Implement delete-all-data.
- Add privacy/about copy.
- Add export integration tests.

Exit criteria:
- App starts locked after setup.
- Idle lock works.
- Passphrase can be changed without data loss.
- Export file is complete and valid JSON.
- Delete-all-data returns app to first-run state.

### Phase 5 — Stats, Accessibility, Packaging, and Release Candidate

Tasks:
- Implement writing streak.
- Implement total entries, total word count, entries this month, top 10 tags.
- Implement `Ctrl+F`.
- Complete empty states and onboarding copy.
- Run accessibility checklist.
- Add app icon, window title, and About screen.
- Create Linux `.deb` and `.AppImage` builds.
- Create Windows `.msi` and `.exe` builds.
- Manual QA pass on supported platforms.

Exit criteria:
- Stats are accurate across restarts.
- Core flows pass keyboard and accessibility checks.
- Installers work on clean target machines.
- Release candidate is ready for stakeholder acceptance testing.

---

## 14. Testing Strategy

### 14.1 Unit Tests

Required coverage:
- HTML sanitization.
- Plain-text extraction.
- Word count calculation.
- Date grouping and streak calculation.
- Tag normalization.
- Input validation.
- Error mapping.
- KDF config serialization.

### 14.2 Repository/Integration Tests

Required coverage:
- Create/update/delete entry.
- Backdating.
- Soft delete exclusion.
- Tag attach/remove/rename/delete.
- FTS trigger sync on insert/update/delete.
- Search filtering combinations.
- Export JSON schema.
- Delete-all-data behavior using temp app data directory.

### 14.3 Performance Tests

Fixtures:
- 10,000 entries.
- Mixed body lengths including 50,000-character entries.
- Mixed moods and tags.

Required checks:
- Search under 300ms.
- Timeline first-page load under 300ms after unlock.
- Export completes without crash.

### 14.4 Manual QA Checklist

Platforms:
- Ubuntu LTS or equivalent mainstream Linux distribution.
- Windows 10 or Windows 11.

Checklist:
- Clean install.
- First-run setup.
- Unlock.
- Create/edit/delete entry.
- Autosave during typing.
- Close app with pending changes.
- Search and filter.
- Tag rename/delete.
- Change passphrase.
- Idle lock.
- Export JSON.
- Delete all data.
- Keyboard-only create/edit/search/export path.
- 150% text scaling sanity check.

---

## 15. Build and Packaging

### 15.1 CI Pipeline

Required jobs:
- Rust format check.
- Rust clippy.
- Rust unit/integration tests.
- Frontend type check.
- Frontend lint.
- Frontend build.
- Tauri build on Linux.
- Tauri build on Windows.

### 15.2 Release Artifacts

Required artifacts:
- Linux `.deb`.
- Linux `.AppImage`.
- Windows `.msi`.
- Windows `.exe` installer.

### 15.3 Versioning

Use semantic versioning:
- MVP internal builds: `0.1.0-alpha.N`.
- Release candidate: `0.1.0-rc.1`.
- MVP release: `0.1.0`.

Database schema version is stored in `app_meta`.
Export schema version is stored in export root field `format`.

---

## 16. Open Questions and Required Decisions

These should be resolved before implementation starts:

1. Which SQLCipher-compatible Rust crate/build configuration will be used?
2. What Argon2id parameters will be used for the target hardware?
3. Which Linux distributions are officially supported for MVP QA?
4. Is ordered list formatting included in MVP if Tiptap provides it with little additional work?
5. Should deleted entries ever be restorable, or is soft delete purely an internal safety mechanism before permanent delete-all-data?
6. What exact app name, icon, and release signing approach will be used?

---

## 17. Development-Ready Definition of Done

A feature is development-complete only when:

1. Backend command, frontend UI, and typed API wrapper are implemented.
2. Input validation and structured errors are implemented.
3. Data is persisted correctly in the encrypted database.
4. Feature works after app restart.
5. Soft-deleted entries are excluded where applicable.
6. Unit and integration tests cover the critical path.
7. Accessibility labels and keyboard behavior are implemented.
8. No sensitive content is logged.
9. Manual QA notes are added for any platform-specific behavior.
10. Acceptance criteria for the feature are met.

---

## 18. MVP Acceptance Test Scenarios

### Scenario 1 — First Entry

Given a fresh install  
When the user sets a passphrase and creates a formatted journal entry with mood and tags  
Then the entry appears in the timeline, remains after restart, and formatting is preserved.

### Scenario 2 — Autosave Protection

Given an existing entry  
When the user edits continuously for more than 5 seconds  
Then the app saves periodically without requiring manual save.  
And when the user closes the app, pending changes are flushed or the user is warned.

### Scenario 3 — Search and Retrieval

Given 10,000 entries  
When the user searches for a term  
Then matching entries appear within 300ms with highlighted snippets.  
And deleted entries do not appear.

### Scenario 4 — Organization

Given entries with multiple tags and moods  
When the user filters by date, tag, and mood  
Then only matching non-deleted entries appear.

### Scenario 5 — Privacy

Given the app has been set up with a passphrase  
When the app launches  
Then journal content is not visible until unlock succeeds.  
And local database contents are not readable as plaintext outside the app.

### Scenario 6 — Export and Delete

Given the user has entries and tags  
When the user exports JSON  
Then the file contains complete non-deleted journal data using `journal_mvp_export_v1`.  
When the user deletes all data  
Then the app returns to first-run setup and prior local journal data is removed.

