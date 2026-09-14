//! Process-wide tracing bootstrap for EMS / IMS binaries.
//!
//! Writes to **stdout** and **`{LOG_DIR}/{server}.log`** (default `logs/<exe>.log`).
//! Levels: `error` < `warn` < `info` < `debug` < `trace`.
//! Set `RUST_LOG` to control verbosity (see [`DEFAULT_FILTER`]).

use std::fs::{create_dir_all, OpenOptions};
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Default when `RUST_LOG` is unset: app crates at **debug**, HTTP at **debug**,
/// noisy SDKs quieter, everything else at **info**.
pub const DEFAULT_FILTER: &str = concat!(
    "ems_server=debug,",
    "ems_upload=debug,",
    "ems_listing=debug,",
    "ems_streaming=debug,",
    "ims_processor=debug,",
    "tower_http=debug,",
    "sqlx=info,",
    "rdkafka=info,",
    "aws_smithy_http_client=warn,",
    "aws_config=warn,",
    "hyper=info,",
    "info"
);

/// Initialize stdout + file logging. Honors `RUST_LOG`, `LOG_DIR`, `LOG_NAME`.
pub fn init() {
    init_with(DEFAULT_FILTER);
}

/// Same as [`init`], but with a custom fallback filter string.
pub fn init_with(default_filter: &str) {
    let name = service_name();
    let path = log_file_path(&name);
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .compact()
        .with_writer(io::stdout);

    match open_log_file(&path) {
        Ok(file) => {
            let file_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_ansi(false)
                .compact()
                .with_writer(Mutex::new(file));
            tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .with(file_layer)
                .init();
            tracing::info!(%name, path = %path.display(), "tracing initialized (stdout + file)");
        }
        Err(e) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(stdout_layer)
                .init();
            tracing::warn!(
                error = %e,
                path = %path.display(),
                "log file unavailable — stdout only"
            );
        }
    }
}

fn service_name() -> String {
    if let Ok(n) = std::env::var("LOG_NAME") {
        let t = n.trim();
        if !t.is_empty() {
            return sanitize_name(t);
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_stem()
                .map(|s| sanitize_name(&s.to_string_lossy()))
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "app".into())
}

fn sanitize_name(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn log_file_path(name: &str) -> PathBuf {
    let dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".into());
    PathBuf::from(dir).join(format!("{name}.log"))
}

fn open_log_file(path: &std::path::Path) -> io::Result<std::fs::File> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    OpenOptions::new().create(true).append(true).open(path)
}
