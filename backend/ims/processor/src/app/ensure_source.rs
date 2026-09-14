use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use shared::VideoId;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::adapters::media_sniff::sniff_video;
use crate::ports::ObjectStore;

/// One download per `(pod, file_id)` so parallel chunk jobs do not clobber `source`.
pub struct SourceCache {
    gates: Mutex<HashMap<VideoId, Arc<Mutex<()>>>>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self {
            gates: Mutex::new(HashMap::new()),
        }
    }

    pub async fn ensure(
        &self,
        objects: &dyn ObjectStore,
        work_dir: &str,
        file_id: VideoId,
        object_key: &str,
    ) -> anyhow::Result<PathBuf> {
        let gate = {
            let mut map = self.gates.lock().await;
            map.entry(file_id)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _ok = gate.lock().await;
        materialize(objects, work_dir, file_id, object_key).await
    }
}

async fn materialize(
    objects: &dyn ObjectStore,
    work_dir: &str,
    file_id: VideoId,
    object_key: &str,
) -> anyhow::Result<PathBuf> {
    let job_dir = PathBuf::from(work_dir).join(file_id.to_string());
    tokio::fs::create_dir_all(&job_dir)
        .await
        .context("create job dir")?;
    if let Some(p) = ready_source(&job_dir).await {
        return Ok(p);
    }
    let part = job_dir.join(format!(".source.{}.part", Uuid::new_v4()));
    objects
        .download_to_path(object_key, &part)
        .await
        .context("download source part")?;
    let kind = sniff_video(&part).await.context("sniff source")?;
    let named = job_dir.join(format!("source.{kind}"));
    if named.is_file() {
        let _ = tokio::fs::remove_file(&part).await;
    } else {
        tokio::fs::rename(&part, &named)
            .await
            .with_context(|| format!("rename {} -> {}", part.display(), named.display()))?;
    }
    write_marker(&job_dir, &named).await?;
    Ok(named)
}

async fn ready_source(job_dir: &std::path::Path) -> Option<PathBuf> {
    let marker = job_dir.join("source.name");
    let name = tokio::fs::read_to_string(&marker).await.ok()?;
    let p = job_dir.join(name.trim());
    p.is_file().then_some(p)
}

pub async fn write_marker(job_dir: &std::path::Path, named: &std::path::Path) -> anyhow::Result<()> {
    let fname = named
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("source");
    tokio::fs::write(job_dir.join("source.name"), fname)
        .await
        .context("write source.name")?;
    Ok(())
}
