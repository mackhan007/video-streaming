//! Chunk-job progress for `GET …/pipeline` (indexed by video + rung + state).

use shared::VideoId;
use sqlx::{PgPool, Row};

use crate::ports::pipeline::{EncodeProgress, PipelineRepoError, RungProgress};

pub(super) async fn load(
    pool: &PgPool,
    video_id: VideoId,
) -> Result<EncodeProgress, PipelineRepoError> {
    let rows = sqlx::query(
        r#"
        SELECT b.pack_ladder,
               j.rung,
               COUNT(*)::int AS total,
               COUNT(*) FILTER (WHERE j.state = 'done')::int AS done
        FROM ims_encode_batches b
        INNER JOIN ims_encode_jobs j ON j.video_id = b.video_id
        WHERE b.video_id = $1
        GROUP BY b.pack_ladder, j.rung
        ORDER BY j.rung
        "#,
    )
    .bind(video_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(|e| PipelineRepoError::Internal(e.into()))?;
    if rows.is_empty() {
        return Ok(EncodeProgress::default());
    }
    let pack_ladder: bool = rows[0].get("pack_ladder");
    let rungs = rows
        .iter()
        .map(|r| RungProgress {
            rung: r.get("rung"),
            done: r.get("done"),
            total: r.get("total"),
        })
        .collect();
    Ok(EncodeProgress {
        pack_ladder,
        rungs,
    })
}
