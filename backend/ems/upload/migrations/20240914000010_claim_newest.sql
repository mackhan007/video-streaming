-- Claim queued jobs newest encoding batch first, then chunk order.
CREATE INDEX ims_encode_jobs_queued_video_chunk
    ON ims_encode_jobs (video_id, chunk_index)
    WHERE state = 'queued';

CREATE INDEX ims_encode_batches_encoding_created
    ON ims_encode_batches (created_at DESC)
    WHERE state = 'encoding';
