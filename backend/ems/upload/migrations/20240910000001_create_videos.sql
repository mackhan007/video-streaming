-- Create video status enum and videos table for EMS upload metadata.

CREATE TYPE video_status AS ENUM (
    'pending',
    'uploaded',
    'processing',
    'ready',
    'failed'
);

CREATE TABLE videos (
    id            UUID PRIMARY KEY,
    status        video_status NOT NULL DEFAULT 'pending',
    title         TEXT,
    content_type  TEXT,
    file_size     BIGINT NOT NULL,
    object_key    TEXT NOT NULL,
    upload_id     TEXT,
    part_size     BIGINT,
    playback_path TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX videos_status_created_at ON videos (status, created_at DESC);