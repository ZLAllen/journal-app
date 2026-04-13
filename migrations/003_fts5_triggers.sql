-- Keep the external-content FTS index in sync with entries writes.

CREATE TRIGGER IF NOT EXISTS entries_ai_fts
AFTER INSERT ON entries
WHEN new.deleted_at IS NULL
BEGIN
    INSERT INTO entries_fts(rowid, body)
    VALUES (new.rowid, new.body);
END;

CREATE TRIGGER IF NOT EXISTS entries_au_fts
AFTER UPDATE OF body, deleted_at ON entries
BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, body)
    SELECT 'delete', old.rowid, old.body
    WHERE old.deleted_at IS NULL;

    INSERT INTO entries_fts(rowid, body)
    SELECT new.rowid, new.body
    WHERE new.deleted_at IS NULL;
END;

CREATE TRIGGER IF NOT EXISTS entries_ad_fts
AFTER DELETE ON entries
BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, body)
    VALUES ('delete', old.rowid, old.body);
END;

INSERT INTO entries_fts(entries_fts) VALUES('rebuild');