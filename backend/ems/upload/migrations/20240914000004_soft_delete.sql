-- Soft delete: retain row + S3 objects; listing should filter deleted_at IS NULL.

ALTER TABLE videos
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS videos_not_deleted_created_at
    ON videos (created_at DESC)
    WHERE deleted_at IS NULL;
