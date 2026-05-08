-- Add sanitized HTML, derived plain text, and word count projections for entries.

ALTER TABLE entries
ADD COLUMN body_html TEXT NOT NULL DEFAULT '';

ALTER TABLE entries
ADD COLUMN body_text TEXT NOT NULL DEFAULT '';

ALTER TABLE entries
ADD COLUMN word_count INTEGER NOT NULL DEFAULT 0;

UPDATE entries
SET body_html = body,
    body_text = body
WHERE body_html = ''
  AND body != '';

DROP TRIGGER IF EXISTS entries_ai_fts;
DROP TRIGGER IF EXISTS entries_au_fts;
DROP TRIGGER IF EXISTS entries_ad_fts;
DROP TABLE IF EXISTS entries_fts;

CREATE VIRTUAL TABLE entries_fts USING fts5(
    body_text,
    content='entries',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER entries_ai_fts
AFTER INSERT ON entries
WHEN new.deleted_at IS NULL
BEGIN
    INSERT INTO entries_fts(rowid, body_text)
    VALUES (new.rowid, new.body_text);
END;

CREATE TRIGGER entries_au_fts
AFTER UPDATE OF body_text, deleted_at ON entries
BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, body_text)
    SELECT 'delete', old.rowid, old.body_text
    WHERE old.deleted_at IS NULL;

    INSERT INTO entries_fts(rowid, body_text)
    SELECT new.rowid, new.body_text
    WHERE new.deleted_at IS NULL;
END;

CREATE TRIGGER entries_ad_fts
AFTER DELETE ON entries
BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, body_text)
    VALUES ('delete', old.rowid, old.body_text);
END;

INSERT INTO entries_fts(entries_fts) VALUES('rebuild');
