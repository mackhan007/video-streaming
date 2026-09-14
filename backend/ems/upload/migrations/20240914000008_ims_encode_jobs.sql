-- Distributed IMS chunk jobs. Any replica claims with SKIP LOCKED.
CREATE TYPE ims_job_state AS ENUM ('queued', 'running', 'done', 'failed');
CREATE TYPE ims_batch_state AS ENUM ('encoding', 'finalizing', 'done', 'failed');

CREATE TABLE ims_encode_batches (
    video_id    UUID PRIMARY KEY REFERENCES videos (id) ON DELETE CASCADE,
    object_key  TEXT NOT NULL,
    total_jobs  INT NOT NULL,
    with_audio  BOOLEAN NOT NULL DEFAULT TRUE,
    state       ims_batch_state NOT NULL DEFAULT 'encoding',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ims_encode_jobs (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    video_id      UUID NOT NULL REFERENCES ims_encode_batches (video_id) ON DELETE CASCADE,
    chunk_index   INT NOT NULL,
    rung          INT NOT NULL,
    start_secs    DOUBLE PRECISION NOT NULL,
    duration_secs DOUBLE PRECISION NOT NULL,
    state         ims_job_state NOT NULL DEFAULT 'queued',
    error         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ims_encode_jobs_video_chunk_rung_uidx UNIQUE (video_id, chunk_index, rung)
);

-- Hot path: claim next queued job (FOR UPDATE SKIP LOCKED ORDER BY id).
CREATE INDEX ims_encode_jobs_queued_id
    ON ims_encode_jobs (id)
    WHERE state = 'queued';

-- Hot path: finalize when no non-done jobs remain for a video.
CREATE INDEX ims_encode_jobs_video_state
    ON ims_encode_jobs (video_id, state);

-- Hot path: find batches still encoding.
CREATE INDEX ims_encode_batches_encoding
    ON ims_encode_batches (updated_at)
    WHERE state = 'encoding';
