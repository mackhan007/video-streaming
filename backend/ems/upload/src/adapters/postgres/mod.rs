//! Postgres video repository (split to stay under the 200-line file limit).

mod encode_progress;
mod pipeline;
mod repo;
mod row;

use anyhow::Context;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::{debug, info};

pub use pipeline::PostgresPipeline;
pub use repo::PostgresVideoRepository;

impl PostgresVideoRepository {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        debug!("opening postgres pool");
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("connect postgres")?;
        info!("postgres pool ready");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        debug!("running sqlx migrations");
        // Re-expand when files in `ems/upload/migrations/` change.
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("run migrations")?;
        info!("sqlx migrations complete");
        Ok(())
    }
}
