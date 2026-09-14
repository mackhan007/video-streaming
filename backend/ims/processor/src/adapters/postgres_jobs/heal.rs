//! Align encode batches with video status after pod restarts.

use sqlx::PgPool;
use tracing::info;

use crate::ports::encode_jobs::EncodeJobError;

/// Stop work on failed/ready videos; requeue `running` rows left by dead pods.
pub(super) async fn heal_encode_queue(pool: &PgPool) -> Result<(), EncodeJobError> {
    let paused = sqlx::query(
        r#"
        UPDATE ims_encode_batches b
        SET state = 'failed', updated_at = NOW()
        FROM videos v
        WHERE b.video_id = v.id
          AND b.state = 'encoding'
          AND v.status <> 'processing'
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    if paused.rows_affected() > 0 {
        info!(batches = paused.rows_affected(), "paused encode batches for non-processing videos");
    }
    let n = sqlx::query(
        r#"
        UPDATE ims_encode_jobs j
        SET state = 'queued', error = NULL, updated_at = NOW()
        FROM videos v
        WHERE j.video_id = v.id
          AND v.status = 'processing'
          AND j.state = 'running'
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    if n.rows_affected() > 0 {
        info!(jobs = n.rows_affected(), "requeued orphaned running encode jobs");
    }
    Ok(())
}
