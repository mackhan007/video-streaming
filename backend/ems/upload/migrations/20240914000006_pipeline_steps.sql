-- Pipeline tracking (separate from videos.status) for UI + async job observability.
CREATE TYPE pipeline_step_name AS ENUM (
    'upload',
    'queue',
    'process',
    'ready',
    'play'
);

CREATE TYPE pipeline_step_state AS ENUM (
    'pending',
    'running',
    'done',
    'failed'
);

CREATE TABLE video_pipeline_steps (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    video_id    UUID NOT NULL REFERENCES videos (id) ON DELETE CASCADE,
    step        pipeline_step_name NOT NULL,
    state       pipeline_step_state NOT NULL DEFAULT 'pending',
    detail      TEXT,
    error       TEXT,
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT video_pipeline_steps_video_step_uidx UNIQUE (video_id, step)
);

-- Hot path: fetch all steps for one video (UI poll).
CREATE INDEX video_pipeline_steps_video_id_idx
    ON video_pipeline_steps (video_id);

-- Hot path: ops / stuck running steps.
CREATE INDEX video_pipeline_steps_state_updated_idx
    ON video_pipeline_steps (state, updated_at);
