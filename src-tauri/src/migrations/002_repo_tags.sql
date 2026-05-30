-- v2 — per-repo tags for grouping. Guarded against pre-migration databases
-- that may already have the column from the earlier ad-hoc migration.
ALTER TABLE repos ADD COLUMN tags TEXT NOT NULL DEFAULT '';
