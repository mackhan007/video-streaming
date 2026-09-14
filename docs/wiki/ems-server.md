# EMS server (unified gateway)

## Entry

| What | Path |
|---|---|
| Bin | `backend/ems/server/src/bin/ems.rs` |
| Lib / router | `backend/ems/server/src/lib.rs` |
| Package | `ems-server` |

Listens on **`EMS_HTTP_PORT`** (default **8080**). Merges upload + listing + streaming routers; owns `/health` and `/ready`.

## Routes

| Method | Path | Source |
|---|---|---|
| `GET` | `/health` | `ems/server/src/lib.rs` |
| `GET` | `/ready` | same → `ems_upload::check_ready` |
| `POST`/`DELETE` | `/uploader/videos…` | REST upload resource (create/complete/abort/delete) |
| `GET` | `/lister/*` | `ems_listing::router` |
| `GET/POST` | `/streamer/*` | `ems_streaming::router` |

HTTP layers (trace + request-id): `ems_upload::apply_http_layers` ← `ems/upload/src/http.rs`.

## Wiring order

1. `Config::from_env()` (upload config)
2. `build_state(&config)` — Postgres migrate, S3, Redis, Kafka
3. Merge routers + layers
4. `axum::serve`

## Related wiki

[ems-upload.md](ems-upload.md) · [ems-stubs.md](ems-stubs.md) · [env-and-ports.md](env-and-ports.md)
