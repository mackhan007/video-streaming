-- Track whether video.uploaded was successfully published (at-least-once recovery).

ALTER TABLE videos
    ADD COLUMN IF NOT EXISTS event_published BOOLEAN NOT NULL DEFAULT false;
