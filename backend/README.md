# Backend (Rust workspace)

Default member: **`ems-server`** (bin **`ems`**).

```bash
# from repo root, with .env loaded (see docs/local-setup.md)
cd backend
cargo run -p ems-server --bin ems
cargo build -p ems-server
```

## Crates

| Package | Binary | Role |
|---|---|---|
| `ems-server` | `ems` | Unified EMS on `EMS_HTTP_PORT` (default 8080) |
| `ems-upload` | `ems-upload` | Upload lib + standalone server (`HTTP_PORT`, default 8085) |
| `ems-listing` | `ems-listing` | Listing stub (`501`) |
| `ems-streaming` | `ems-streaming` | Streaming stub (`501`) |
| `ims-processor` | `ims-processor` | Worker stub (health only) |
| `shared` | — | `VideoId`, `VideoStatus`, `VideoUploaded`, `logging::init` |

## Upload internals

Hexagonal layout under `ems/upload/`:

- `api/` — HTTP DTOs, routes, errors  
- `app/` — `GetUploadUrl`, `CompleteUpload`  
- `domain/` — pure types  
- `ports/` — traits  
- `adapters/` — Postgres, Redis, S3, Kafka  
- `migrations/` — `videos` + hot-path indexes (applied on startup)

See root [README.md](../README.md), [docs/local-setup.md](../docs/local-setup.md), and **[docs/wiki/INDEX.md](../docs/wiki/INDEX.md)**.

License: **MIT** (workspace `license = "MIT"`; see repo root [LICENSE](../LICENSE)).
