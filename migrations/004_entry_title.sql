-- Add a dedicated title field for entries.

ALTER TABLE entries
ADD COLUMN title TEXT NOT NULL DEFAULT '';
