use crate::app::{
    AbortUpload, CompleteUpload, CompleteUploadError, GetUploadUrl, GetUploadUrlInput,
    SoftDeleteVideo, UploadLimits,
};
use crate::fakes::{FakeEvents, FakeObjects, FakeSessions, FakeVideos};

fn limits() -> UploadLimits {
    UploadLimits {
        part_size: 1_000,
        max_upload_bytes: 10_000,
        max_title_chars: 32,
        allowed_content_types: vec!["video/mp4".into()],
        presign_ttl_secs: 60,
    }
}

#[tokio::test]
async fn get_upload_url_single_persists_session() {
    let videos = FakeVideos::default();
    let objects = FakeObjects::default();
    let sessions = FakeSessions::default();
    let uc = GetUploadUrl::new(&videos, &objects, &sessions, limits());
    let out = uc
        .execute(GetUploadUrlInput {
            file_size: 500,
            title: Some("t".into()),
            content_type: Some("video/mp4".into()),
        })
        .await
        .unwrap();
    assert_eq!(out.parts.len(), 1);
    assert!(sessions.map.lock().unwrap().contains_key(&out.file_id));
    assert!(videos.rows.lock().unwrap().contains_key(&out.file_id));
}

#[tokio::test]
async fn complete_checks_size_and_publishes() {
    let videos = FakeVideos::default();
    let objects = FakeObjects::default();
    let sessions = FakeSessions::default();
    let events = FakeEvents::default();
    let uc = GetUploadUrl::new(&videos, &objects, &sessions, limits());
    let out = uc
        .execute(GetUploadUrlInput {
            file_size: 100,
            title: None,
            content_type: Some("video/mp4".into()),
        })
        .await
        .unwrap();
    objects
        .heads
        .lock()
        .unwrap()
        .insert(out.object_key.clone(), 100);

    let done = CompleteUpload::new(&videos, &objects, &sessions, &events)
        .execute(out.file_id)
        .await
        .unwrap();
    assert_eq!(done.status.as_str(), "uploaded");
    assert_eq!(events.published.lock().unwrap().len(), 1);
    assert!(videos.rows.lock().unwrap().get(&out.file_id).unwrap().event_published);

    // idempotent replay does not double-publish
    CompleteUpload::new(&videos, &objects, &sessions, &events)
        .execute(out.file_id)
        .await
        .unwrap();
    assert_eq!(events.published.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn complete_rejects_size_mismatch() {
    let videos = FakeVideos::default();
    let objects = FakeObjects::default();
    let sessions = FakeSessions::default();
    let events = FakeEvents::default();
    let out = GetUploadUrl::new(&videos, &objects, &sessions, limits())
        .execute(GetUploadUrlInput {
            file_size: 100,
            title: None,
            content_type: None,
        })
        .await
        .unwrap();
    objects
        .heads
        .lock()
        .unwrap()
        .insert(out.object_key.clone(), 50);
    let err = CompleteUpload::new(&videos, &objects, &sessions, &events)
        .execute(out.file_id)
        .await
        .unwrap_err();
    assert!(matches!(err, CompleteUploadError::SizeMismatch { .. }));
}

#[tokio::test]
async fn abort_marks_failed() {
    let videos = FakeVideos::default();
    let objects = FakeObjects::default();
    let sessions = FakeSessions::default();
    let out = GetUploadUrl::new(&videos, &objects, &sessions, limits())
        .execute(GetUploadUrlInput {
            file_size: 5000,
            title: None,
            content_type: Some("video/mp4".into()),
        })
        .await
        .unwrap();
    assert!(out.upload_id.is_some());
    let aborted = AbortUpload::new(&videos, &objects, &sessions)
        .execute(out.file_id)
        .await
        .unwrap();
    assert_eq!(aborted.status.as_str(), "failed");
    assert!(!objects.aborted.lock().unwrap().is_empty());
}

#[tokio::test]
async fn soft_delete_is_idempotent_and_blocks_complete() {
    let videos = FakeVideos::default();
    let objects = FakeObjects::default();
    let sessions = FakeSessions::default();
    let events = FakeEvents::default();
    let out = GetUploadUrl::new(&videos, &objects, &sessions, limits())
        .execute(GetUploadUrlInput {
            file_size: 100,
            title: None,
            content_type: Some("video/mp4".into()),
        })
        .await
        .unwrap();
    objects
        .heads
        .lock()
        .unwrap()
        .insert(out.object_key.clone(), 100);

    let del = SoftDeleteVideo::new(&videos, &objects, &sessions)
        .execute(out.file_id)
        .await
        .unwrap();
    assert!(videos.rows.lock().unwrap().get(&out.file_id).unwrap().is_deleted());

    let again = SoftDeleteVideo::new(&videos, &objects, &sessions)
        .execute(out.file_id)
        .await
        .unwrap();
    assert_eq!(del.deleted_at, again.deleted_at);

    let err = CompleteUpload::new(&videos, &objects, &sessions, &events)
        .execute(out.file_id)
        .await
        .unwrap_err();
    assert!(matches!(err, CompleteUploadError::Deleted(_)));
}
