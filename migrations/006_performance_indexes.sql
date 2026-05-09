-- Add indexes for timeline ordering and common filters.

CREATE INDEX IF NOT EXISTS idx_entries_timeline
ON entries(deleted_at, pinned DESC, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_entries_created_filter
ON entries(deleted_at, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_entries_mood_filter
ON entries(deleted_at, mood, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_entry_tags_tag_entry
ON entry_tags(tag_id, entry_id);
