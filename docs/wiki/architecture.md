# Architecture

## Workspace

- Root: `backend/Cargo.toml`
- `default-members = ["ems/server"]`
- Logging bootstrap: `backend/shared/src/logging.rs` → `shared::logging::init()`  
  → stdout **and** `{LOG_DIR}/{LOG_NAME|exe}.log` (default `logs/ems.log`, `logs/ims-processor.log`, …)

## Crates → binaries

| Crate path | Package | Binary | Default port |
|---|---|---|---|
| `backend/ems/server` | `ems-server` | `ems` | `EMS_HTTP_PORT` 8080 |
| `backend/ems/upload` | `ems-upload` | `ems-upload` | `HTTP_PORT` 8085 |
| `backend/ems/listing` | `ems-listing` | `ems-listing` | 8086 |
| `backend/ems/streaming` | `ems-streaming` | `ems-streaming` | 8087 |
| `backend/ims/processor` | `ims-processor` | `ims-processor` | 8088 |
| `backend/shared` | `shared` | (lib) | — |

## Hexagonal layers (upload — template for others)

```
api/  →  app/  →  ports/  ←  adapters/
         ↑
      domain/   (no IO)
```

Composition roots only: `ems/upload/src/lib.rs` (`build_state`), `ems/server/src/lib.rs`, bins under `src/bin/`.

## Shared types

| Type | File |
|---|---|
| `VideoId` | `backend/shared/src/video_id.rs` |
| `VideoStatus` | `backend/shared/src/video_status.rs` |
| `VideoUploaded` | `backend/shared/src/events.rs` |

## Run

```bash
cd backend && cargo run -p ems-server --bin ems
```

System cargo (`~/.cargo/bin`). macOS: `brew install cmake` for rdkafka.

Local: `IMS_WORKERS` extra IMS binaries ([frontend.md](frontend.md) / `scripts/dev.sh`). K8s: [helm.md](helm.md).
