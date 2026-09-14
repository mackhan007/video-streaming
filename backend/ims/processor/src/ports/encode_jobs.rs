use shared::VideoId;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChunkSpec {
    pub index: i32,
    pub start_secs: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone)]
pub struct EncodeJob {
    pub id: Uuid,
    pub video_id: VideoId,
    pub object_key: String,
    pub chunk_index: i32,
    pub rung: i32,
    pub start_secs: f64,
    pub duration_secs: f64,
    pub with_audio: bool,
    /// One FFmpeg per chunk encodes every ABR rung (new batches).
    pub pack_ladder: bool,
}

#[derive(Debug, Clone)]
pub struct EncodeBatch {
    pub video_id: VideoId,
    pub with_audio: bool,
}

#[derive(Debug, Error)]
pub enum EncodeJobError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait EncodeJobRepository: Send + Sync {
    async fn has_batch(&self, video_id: VideoId) -> Result<bool, EncodeJobError>;

    async fn revive_failed_batch(&self, video_id: VideoId) -> Result<bool, EncodeJobError>;

    /// Collapse old per-rung jobs into one packed job per chunk.
    async fn upgrade_legacy_batches(&self) -> Result<u64, EncodeJobError>;

    /// Pause batches for non-processing videos; requeue orphaned `running` jobs.
    async fn heal_encode_queue(&self) -> Result<(), EncodeJobError>;

    /// Insert batch + jobs. `Ok(false)` if this video was already queued.
    async fn enqueue(
        &self,
        video_id: VideoId,
        object_key: &str,
        with_audio: bool,
        chunks: &[ChunkSpec],
        n_rungs: i32,
    ) -> Result<bool, EncodeJobError>;

    async fn claim_queued(&self) -> Result<Option<EncodeJob>, EncodeJobError>;

    async fn mark_done(&self, id: Uuid) -> Result<(), EncodeJobError>;

    async fn fail_batch(&self, video_id: VideoId, error: &str) -> Result<(), EncodeJobError>;

    async fn ready_to_finalize(&self, limit: i64) -> Result<Vec<VideoId>, EncodeJobError>;

    async fn claim_finalize(
        &self,
        video_id: VideoId,
    ) -> Result<Option<EncodeBatch>, EncodeJobError>;

    async fn mark_batch_done(&self, video_id: VideoId) -> Result<(), EncodeJobError>;

    async fn chunk_indexes(&self, video_id: VideoId) -> Result<Vec<i32>, EncodeJobError>;
}
