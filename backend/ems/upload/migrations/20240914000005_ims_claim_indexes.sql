-- Hot path: IMS claims uploaded rows by status + freshness.
CREATE INDEX IF NOT EXISTS videos_uploaded_updated_at
    ON videos (updated_at)
    WHERE status = 'uploaded' AND deleted_at IS NULL;

-- Hot path: reclaim stuck processing jobs.
CREATE INDEX IF NOT EXISTS videos_processing_updated_at
    ON videos (updated_at)
    WHERE status = 'processing' AND deleted_at IS NULL;
