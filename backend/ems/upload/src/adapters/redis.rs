use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use shared::VideoId;
use tracing::{debug, error, info};

use crate::domain::UploadSession;
use crate::ports::sessions::{SessionStore, SessionStoreError};

pub struct RedisSessionStore {
    conn: ConnectionManager,
    ttl_secs: u64,
}

impl RedisSessionStore {
    pub async fn connect(redis_url: &str, ttl_secs: u64) -> anyhow::Result<Self> {
        debug!(%redis_url, ttl_secs, "connecting redis");
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        info!(ttl_secs, "redis connection manager ready");
        Ok(Self { conn, ttl_secs })
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn put(&self, session: &UploadSession) -> Result<(), SessionStoreError> {
        let key = UploadSession::redis_key(session.file_id);
        debug!(file_id = %session.file_id, %key, "session put");
        let payload =
            serde_json::to_string(session).map_err(|e| SessionStoreError::Internal(e.into()))?;
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, payload, self.ttl_secs)
            .await
            .map_err(|e| {
                error!(error = %e, file_id = %session.file_id, "session put failed");
                SessionStoreError::Internal(e.into())
            })?;
        Ok(())
    }

    async fn get(&self, file_id: VideoId) -> Result<Option<UploadSession>, SessionStoreError> {
        let key = UploadSession::redis_key(file_id);
        debug!(%file_id, %key, "session get");
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| {
                error!(error = %e, %file_id, "session get failed");
                SessionStoreError::Internal(e.into())
            })?;
        match raw {
            Some(s) => {
                let session = serde_json::from_str(&s)
                    .map_err(|e| SessionStoreError::Internal(e.into()))?;
                Ok(Some(session))
            }
            None => {
                debug!(%file_id, "session miss");
                Ok(None)
            }
        }
    }

    async fn delete(&self, file_id: VideoId) -> Result<(), SessionStoreError> {
        let key = UploadSession::redis_key(file_id);
        debug!(%file_id, %key, "session delete");
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| {
                error!(error = %e, %file_id, "session delete failed");
                SessionStoreError::Internal(e.into())
            })?;
        Ok(())
    }

    async fn ping(&self) -> Result<(), SessionStoreError> {
        let mut conn = self.conn.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| SessionStoreError::Internal(e.into()))?;
        Ok(())
    }
}