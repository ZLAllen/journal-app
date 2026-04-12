-- Full-text search index using SQLite FTS5

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    body,
    content='entries',
    content_rowid='rowid'
);

-- Insert existing entries into the FTS table (if any)
-- This is safe to run even if the table is empty
INSERT OR IGNORE INTO entries_fts(rowid, body)
SELECT rowid, body FROM entries;
