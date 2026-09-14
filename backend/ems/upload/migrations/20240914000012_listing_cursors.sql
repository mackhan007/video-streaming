-- Keyset listing: (created_at, id) for live rows and ready catalog.
-- Matches GET /lister/videos and GET /lister/links ORDER BY created_at DESC, id DESC.

CREATE INDEX IF NOT EXISTS videos_live_created_id
    ON videos (created_at DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS videos_ready_live_created_id
    ON videos (created_at DESC, id DESC)
    WHERE status = 'ready' AND deleted_at IS NULL;
