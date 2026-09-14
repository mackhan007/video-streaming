//! In-memory port fakes for use-case unit tests.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use shared::VideoId;

use crate::domain::{UploadSession, Video};
use crate::ports::objects::{CompletedPart, ObjectStore, ObjectStoreError};
use crate::ports::sessions::{SessionStore, SessionStoreError};
use crate::ports::videos::{VideoRepoError, VideoRepository};

#[derive(Default)]
pub struct FakeVideos {
    pub rows: Mutex<HashMap<VideoId, Video>>,
}

#[async_trait]
impl VideoRepository for FakeVideos {
    async fn insert(&self, video: &Video) -> Result<(), VideoRepoError> {
        self.rows.lock().unwrap().insert(video.id, video.clone());
        Ok(())
    }

    async fn get(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        self.rows
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(VideoRepoError::NotFound(id))
    }

    async fn mark_uploaded(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        let mut g = self.rows.lock().unwrap();
        let v = g.get_mut(&id).ok_or(VideoRepoError::NotFound(id))?;
        if v.status == shared::VideoStatus::Pending {
            v.status = shared::VideoStatus::Uploaded;
        }
        Ok(v.clone())
    }

    async fn mark_event_published(&self, id: VideoId) -> Result<(), VideoRepoError> {
        let mut g = self.rows.lock().unwrap();
        let v = g.get_mut(&id).ok_or(VideoRepoError::NotFound(id))?;
        v.event_published = true;
        Ok(())
    }

    async fn mark_failed(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        let mut g = self.rows.lock().unwrap();
        let v = g.get_mut(&id).ok_or(VideoRepoError::NotFound(id))?;
        if v.status == shared::VideoStatus::Pending {
            v.status = shared::VideoStatus::Failed;
        }
        Ok(v.clone())
    }

    async fn requeue_failed(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        let mut g = self.rows.lock().unwrap();
        let v = g.get_mut(&id).ok_or(VideoRepoError::NotFound(id))?;
        if !matches!(
            v.status,
            shared::VideoStatus::Failed
                | shared::VideoStatus::Processing
                | shared::VideoStatus::Uploaded
        ) {
            return Err(VideoRepoError::NotFound(id));
        }
        v.status = shared::VideoStatus::Uploaded;
        v.playback_path = None;
        v.event_published = false;
        Ok(v.clone())
    }

    async fn soft_delete(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        let mut g = self.rows.lock().unwrap();
        let v = g.get_mut(&id).ok_or(VideoRepoError::NotFound(id))?;
        if v.deleted_at.is_none() {
            v.deleted_at = Some(chrono::Utc::now());
        }
        Ok(v.clone())
    }

    async fn ping(&self) -> Result<(), VideoRepoError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeObjects {
    pub heads: Mutex<HashMap<String, u64>>,
    pub multiparts: Mutex<HashMap<String, String>>,
    pub aborted: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl ObjectStore for FakeObjects {
    async fn create_multipart_upload(
        &self,
        object_key: &str,
        _content_type: Option<&str>,
    ) -> Result<String, ObjectStoreError> {
        let id = format!("mpu-{object_key}");
        self.multiparts
            .lock()
            .unwrap()
            .insert(object_key.to_string(), id.clone());
        Ok(id)
    }

    async fn presign_put_object(
        &self,
        object_key: &str,
        _content_type: Option<&str>,
    ) -> Result<String, ObjectStoreError> {
        Ok(format!("https://example.test/put/{object_key}"))
    }

    async fn presign_upload_part(
        &self,
        object_key: &str,
        upload_id: &str,
        part_number: i32,
    ) -> Result<String, ObjectStoreError> {
        Ok(format!(
            "https://example.test/part/{object_key}/{upload_id}/{part_number}"
        ))
    }

    async fn list_parts(
        &self,
        _object_key: &str,
        _upload_id: &str,
    ) -> Result<Vec<CompletedPart>, ObjectStoreError> {
        Ok(vec![CompletedPart {
            part_number: 1,
            etag: "\"etag\"".into(),
        }])
    }

    async fn complete_multipart_upload(
        &self,
        object_key: &str,
        _upload_id: &str,
        _parts: Vec<CompletedPart>,
    ) -> Result<(), ObjectStoreError> {
        self.heads.lock().unwrap().insert(object_key.to_string(), 0);
        Ok(())
    }

    async fn abort_multipart_upload(
        &self,
        object_key: &str,
        upload_id: &str,
    ) -> Result<(), ObjectStoreError> {
        self.aborted
            .lock()
            .unwrap()
            .push((object_key.to_string(), upload_id.to_string()));
        self.multiparts.lock().unwrap().remove(object_key);
        Ok(())
    }

    async fn head_object(&self, object_key: &str) -> Result<Option<u64>, ObjectStoreError> {
        Ok(self.heads.lock().unwrap().get(object_key).copied())
    }

    async fn ping(&self) -> Result<(), ObjectStoreError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeSessions {
    pub map: Mutex<HashMap<VideoId, UploadSession>>,
}

#[async_trait]
impl SessionStore for FakeSessions {
    async fn put(&self, session: &UploadSession) -> Result<(), SessionStoreError> {
        self.map.lock().unwrap().insert(session.file_id, session.clone());
        Ok(())
    }

    async fn get(&self, file_id: VideoId) -> Result<Option<UploadSession>, SessionStoreError> {
        Ok(self.map.lock().unwrap().get(&file_id).cloned())
    }

    async fn delete(&self, file_id: VideoId) -> Result<(), SessionStoreError> {
        self.map.lock().unwrap().remove(&file_id);
        Ok(())
    }

    async fn ping(&self) -> Result<(), SessionStoreError> {
        Ok(())
    }
}
