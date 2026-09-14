-- ABR ladder rungs as first-class pipeline steps (UI + tracking).
ALTER TYPE pipeline_step_name ADD VALUE IF NOT EXISTS 'hls_360';
ALTER TYPE pipeline_step_name ADD VALUE IF NOT EXISTS 'hls_720';
ALTER TYPE pipeline_step_name ADD VALUE IF NOT EXISTS 'hls_1080';
