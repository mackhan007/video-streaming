use anyhow::Context;
use shared::VideoId;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::ports::encode_jobs::{ChunkSpec, EncodeJobError};

pub(super) async fn insert_jobs(
    pool: &PgPool,
    video_id: VideoId,
    chunks: &[ChunkSpec],
    n_rungs: i32,
) -> Result<(), EncodeJobError> {
    if chunks.is_empty() || n_rungs <= 0 {
        return Ok(());
    }
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO ims_encode_jobs (video_id, chunk_index, rung, start_secs, duration_secs) ",
    );
    qb.push_values(expanded(chunks, n_rungs), |mut row, spec| {
        row.push_bind(video_id.as_uuid())
            .push_bind(spec.0)
            .push_bind(spec.1)
            .push_bind(spec.2)
            .push_bind(spec.3);
    });
    qb.push(" ON CONFLICT (video_id, chunk_index, rung) DO NOTHING");
    qb.build()
        .execute(pool)
        .await
        .context("insert encode jobs")
        .map_err(EncodeJobError::Internal)?;
    Ok(())
}

fn expanded(chunks: &[ChunkSpec], n_rungs: i32) -> Vec<(i32, i32, f64, f64)> {
    let mut out = Vec::with_capacity(chunks.len() * n_rungs as usize);
    for c in chunks {
        for rung in 0..n_rungs {
            out.push((c.index, rung, c.start_secs, c.duration_secs));
        }
    }
    out
}
