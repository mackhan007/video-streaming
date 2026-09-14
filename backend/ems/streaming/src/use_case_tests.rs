use shared::VideoId;
use uuid::Uuid;

use crate::app::{GetUserState, SaveUserState, SaveUserStateError};
use crate::domain::WatchProgress;
use crate::fakes::FakeWatch;

fn progress(pos: f64, dur: Option<f64>) -> WatchProgress {
    WatchProgress {
        viewer_id: Uuid::nil(),
        file_id: VideoId::new(),
        position_secs: pos,
        duration_secs: dur,
    }
}

#[tokio::test]
async fn save_then_get_round_trip() {
    let store = FakeWatch::default();
    let saved = SaveUserState::new(&store)
        .execute(progress(12.5, Some(120.0)))
        .await
        .unwrap();
    let got = GetUserState::new(&store)
        .execute(saved.viewer_id, saved.file_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.position_secs, 12.5);
    assert_eq!(got.duration_secs, Some(120.0));
}

#[tokio::test]
async fn get_miss_is_none() {
    let store = FakeWatch::default();
    let got = GetUserState::new(&store)
        .execute(Uuid::nil(), VideoId::new())
        .await
        .unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn rejects_negative_position() {
    let store = FakeWatch::default();
    let err = SaveUserState::new(&store)
        .execute(progress(-1.0, None))
        .await
        .unwrap_err();
    assert!(matches!(err, SaveUserStateError::BadPosition));
}

#[tokio::test]
async fn rejects_non_finite_duration() {
    let store = FakeWatch::default();
    let err = SaveUserState::new(&store)
        .execute(progress(1.0, Some(f64::NAN)))
        .await
        .unwrap_err();
    assert!(matches!(err, SaveUserStateError::BadDuration));
}
