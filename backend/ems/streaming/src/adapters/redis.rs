use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use shared::VideoId;
use tracing::{debug, error};
use uuid::Uuid;

use crate::domain::WatchProgress;
use crate::ports::{WatchProgressStore, WatchStoreError};

pub struct RedisWatchStore {
    conn: ConnectionManager,
    ttl_secs: u64,
}

impl RedisWatchStore {
    pub async fn connect(redis_url: &str, ttl_secs: u64) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn, ttl_secs })
    }
}

#[async_trait::async_trait]
impl WatchProgressStore for RedisWatchStore {
    async fn put(&self, progress: &WatchProgress) -> Result<(), WatchStoreError> {
        let key = WatchProgress::redis_key(progress.viewer_id, progress.file_id);
        debug!(%key, "watch put");
        let payload =
            serde_json::to_string(progress).map_err(|e| WatchStoreError::Internal(e.into()))?;
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, payload, self.ttl_secs)
            .await
            .map_err(|e| {
                error!(error = %e, file_id = %progress.file_id, "watch put failed");
                WatchStoreError::Internal(e.into())
            })?;
        Ok(())
    }

    async fn get(
        &self,
        viewer_id: Uuid,
        file_id: VideoId,
    ) -> Result<Option<WatchProgress>, WatchStoreError> {
        let key = WatchProgress::redis_key(viewer_id, file_id);
        debug!(%key, "watch get");
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(key).await.map_err(|e| {
            error!(error = %e, %file_id, "watch get failed");
            WatchStoreError::Internal(e.into())
        })?;
        match raw {
            Some(s) => {
                let progress =
                    serde_json::from_str(&s).map_err(|e| WatchStoreError::Internal(e.into()))?;
                Ok(Some(progress))
            }
            None => Ok(None),
        }
    }
}
