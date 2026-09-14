//! Convert legacy per-rung encode jobs into one packed job per chunk.

use sqlx::{PgPool, Postgres, Transaction};
use tracing::info;
use uuid::Uuid;

use crate::ports::encode_jobs::EncodeJobError;

pub(super) async fn upgrade_legacy_batches(pool: &PgPool) -> Result<u64, EncodeJobError> {
    let mut n = 0u64;
    loop {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| EncodeJobError::Internal(e.into()))?;
        let Some(video_id) = claim_legacy(&mut tx).await? else {
            tx.commit()
                .await
                .map_err(|e| EncodeJobError::Internal(e.into()))?;
            break;
        };
        rewrite_jobs(&mut tx, video_id).await?;
        tx.commit()
            .await
            .map_err(|e| EncodeJobError::Internal(e.into()))?;
        n += 1;
        info!(%video_id, "upgraded encode batch to packed ladder");
    }
    Ok(n)
}

async fn claim_legacy(tx: &mut Transaction<'_, Postgres>) -> Result<Option<Uuid>, EncodeJobError> {
    sqlx::query_scalar(
        r#"
        SELECT b.video_id
        FROM ims_encode_batches b
        INNER JOIN videos v ON v.id = b.video_id AND v.status = 'processing'
        WHERE b.pack_ladder = FALSE
        FOR UPDATE OF b SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))
}

async fn rewrite_jobs(
    tx: &mut Transaction<'_, Postgres>,
    video_id: Uuid,
) -> Result<(), EncodeJobError> {
    sqlx::query(
        r#"
        WITH keep AS (
            SELECT DISTINCT ON (chunk_index) id
            FROM ims_encode_jobs
            WHERE video_id = $1
            ORDER BY chunk_index, rung
        )
        DELETE FROM ims_encode_jobs
        WHERE video_id = $1
          AND id NOT IN (SELECT id FROM keep)
        "#,
    )
    .bind(video_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    sqlx::query(
        r#"
        UPDATE ims_encode_jobs
        SET rung = 0, state = 'queued', error = NULL, updated_at = NOW()
        WHERE video_id = $1
        "#,
    )
    .bind(video_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    sqlx::query(
        r#"
        UPDATE ims_encode_batches
        SET pack_ladder = TRUE,
            state = CASE
                WHEN state = 'failed' THEN 'encoding'::ims_batch_state
                ELSE state
            END,
            total_jobs = (
                SELECT COUNT(*)::int FROM ims_encode_jobs WHERE video_id = $1
            ),
            updated_at = NOW()
        WHERE video_id = $1
        "#,
    )
    .bind(video_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| EncodeJobError::Internal(e.into()))?;
    Ok(())
}
