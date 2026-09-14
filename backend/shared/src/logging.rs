//! Process-wide tracing bootstrap for EMS / IMS binaries.
//!
//! Levels: `error` < `warn` < `info` < `debug` < `trace`.
//! Set `RUST_LOG` to control verbosity (see [`DEFAULT_FILTER`]).

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

/// Initialize stdout logging. Honors `RUST_LOG` when set.
///
/// Examples:
/// - `RUST_LOG=debug` — everything at debug
/// - `RUST_LOG=error` — errors only
/// - `RUST_LOG=ems_upload=trace,info` — deep upload traces
pub fn init() {
    init_with(DEFAULT_FILTER);
}

/// Same as [`init`], but with a custom fallback filter string.
pub fn init_with(default_filter: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .compact()
        .init();

    tracing::debug!(filter = %default_filter, "tracing initialized (override with RUST_LOG)");
}
