use thiserror::Error;

use crate::domain::WatchProgress;
use crate::ports::WatchProgressStore;

pub const MAX_POSITION_SECS: f64 = 86_400.0;

#[derive(Debug, Error)]
pub enum SaveUserStateError {
    #[error("position_secs must be finite and between 0 and 86400")]
    BadPosition,
    #[error("duration_secs must be finite and between 0 and 86400")]
    BadDuration,
    #[error(transparent)]
    Store(#[from] crate::ports::WatchStoreError),
}

pub struct SaveUserState<'a> {
    watch: &'a dyn WatchProgressStore,
}

impl<'a> SaveUserState<'a> {
    pub fn new(watch: &'a dyn WatchProgressStore) -> Self {
        Self { watch }
    }

    pub async fn execute(
        &self,
        progress: WatchProgress,
    ) -> Result<WatchProgress, SaveUserStateError> {
        validate(&progress)?;
        self.watch.put(&progress).await?;
        Ok(progress)
    }
}

fn validate(p: &WatchProgress) -> Result<(), SaveUserStateError> {
    if !valid_secs(p.position_secs) {
        return Err(SaveUserStateError::BadPosition);
    }
    if let Some(d) = p.duration_secs {
        if !valid_secs(d) {
            return Err(SaveUserStateError::BadDuration);
        }
    }
    Ok(())
}

fn valid_secs(v: f64) -> bool {
    v.is_finite() && (0.0..=MAX_POSITION_SECS).contains(&v)
}
