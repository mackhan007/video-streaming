-- Hot path: pipeline poll groups encode jobs by (video, rung, state).
CREATE INDEX ims_encode_jobs_video_rung_state
    ON ims_encode_jobs (video_id, rung, state);
