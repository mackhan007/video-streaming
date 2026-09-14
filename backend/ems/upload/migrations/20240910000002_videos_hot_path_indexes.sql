-- Hot-path indexes for listing, streaming, and IMS (1M TPS readiness).
-- PK on videos.id already covers point gets/updates by file_id.

-- Listing: ready (or status-filtered) feeds ordered by created_at / cursor.
CREATE INDEX IF NOT EXISTS videos_ready_created_at
    ON videos (created_at DESC)
    WHERE status = 'ready';

-- IMS / ops: pending or processing queues by updated_at.
CREATE INDEX IF NOT EXISTS videos_status_updated_at
    ON videos (status, updated_at);

-- Unique object key (raw S3 path) for idempotent storage lookups.
CREATE UNIQUE INDEX IF NOT EXISTS videos_object_key_uidx
    ON videos (object_key);

-- Streaming / CDN path resolution when clients key by playback_path.
CREATE INDEX IF NOT EXISTS videos_playback_path
    ON videos (playback_path)
    WHERE playback_path IS NOT NULL;
