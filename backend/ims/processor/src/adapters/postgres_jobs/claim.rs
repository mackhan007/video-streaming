use shared::VideoId;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::ports::encode_jobs::{EncodeBatch, EncodeJob, EncodeJobError};

pub(super) async fn claim_queued(pool: &PgPool) -> Result<Option<EncodeJob>, EncodeJobError> {
    let row = sqlx::query(
        r#"
        WITH cte AS (
            SELECT j.id
            FROM ims_encode_jobs j
            INNER JOIN ims_encode_batches b
                ON b.video_id = j.video_id AND b.state = 'encoding'
            INNER JOIN videos v
                ON v.id = j.video_id AND v.status = 'processing'
            WHERE j.state = 'queued'
            ORDER BY b.created_at DESC, j.chunk_index
            FOR UPDATE OF j SKIP LOCKED
            LIMIT 1
        )
        UPDATE ims_encode_jobs j
        SET state = 'running', updated_at = NOW()
        FROM cte, ims_encode_batches b
        WHERE j.id = cte.id AND b.video_id = j.video_id
        RETURNING j.id, j.video_id, j.chunk_index, j.rung, j.start_secs,
                  j.duration_secs, b.object_key, b.with_audio, b.pack_ladder
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    Ok(row.map(row_to_job))
}

pub(super) async fn fail_batch(
    pool: &PgPool,
    video_id: VideoId,
    error: &str,
) -> Result<(), EncodeJobError> {
    sqlx::query(
        r#"
        UPDATE ims_encode_jobs
        SET state = 'failed', error = $2, updated_at = NOW()
        WHERE video_id = $1 AND state IN ('queued', 'running')
        "#,
    )
    .bind(video_id.as_uuid())
    .bind(error)
    .execute(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    sqlx::query(
        r#"
        UPDATE ims_encode_batches
        SET state = 'failed', updated_at = NOW()
        WHERE video_id = $1
        "#,
    )
    .bind(video_id.as_uuid())
    .execute(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    Ok(())
}

pub(super) async fn ready_to_finalize(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<VideoId>, EncodeJobError> {
    let rows = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT b.video_id
        FROM ims_encode_batches b
        WHERE b.state = 'encoding'
          AND NOT EXISTS (
              SELECT 1 FROM ims_encode_jobs j
              WHERE j.video_id = b.video_id AND j.state <> 'done'
          )
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    Ok(rows.into_iter().map(VideoId::from).collect())
}

pub(super) async fn claim_finalize(
    pool: &PgPool,
    video_id: VideoId,
) -> Result<Option<EncodeBatch>, EncodeJobError> {
    let row = sqlx::query(
        r#"
        UPDATE ims_encode_batches
        SET state = 'finalizing', updated_at = NOW()
        WHERE video_id = $1 AND state = 'encoding'
            RETURNING video_id, with_audio
        "#,
    )
    .bind(video_id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    Ok(row.map(|r| EncodeBatch {
        video_id: VideoId::from(r.get::<Uuid, _>("video_id")),
        with_audio: r.get("with_audio"),
    }))
}

pub(super) async fn revive_failed_batch(
    pool: &PgPool,
    video_id: VideoId,
) -> Result<bool, EncodeJobError> {
    let updated = sqlx::query(
        r#"
        UPDATE ims_encode_batches
        SET state = 'encoding', updated_at = NOW()
        WHERE video_id = $1 AND state = 'failed'
        "#,
    )
    .bind(video_id.as_uuid())
    .execute(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    if updated.rows_affected() == 0 {
        return Ok(false);
    }
    sqlx::query(
        r#"
        UPDATE ims_encode_jobs
        SET state = 'queued', error = NULL, updated_at = NOW()
        WHERE video_id = $1 AND state IN ('failed', 'running')
        "#,
    )
    .bind(video_id.as_uuid())
    .execute(pool)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    Ok(true)
}

fn row_to_job(r: sqlx::postgres::PgRow) -> EncodeJob {
    EncodeJob {
        id: r.get("id"),
        video_id: VideoId::from(r.get::<Uuid, _>("video_id")),
        object_key: r.get("object_key"),
        chunk_index: r.get("chunk_index"),
        rung: r.get("rung"),
        start_secs: r.get("start_secs"),
        duration_secs: r.get("duration_secs"),
        with_audio: r.get("with_audio"),
        pack_ladder: r.get("pack_ladder"),
    }
}
