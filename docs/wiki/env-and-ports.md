# Env and ports

Templates: `docs/local-setup/.env.template` · loader: `docs/local-setup/export.sh`  
Config parse: `backend/ems/upload/src/config.rs` · IMS: `backend/ims/processor/src/config.rs`

## App ports

| Var | Default | Who |
|---|---|---|
| `EMS_HTTP_PORT` | 8080 | `ems` gateway |
| `HTTP_PORT` | 8085 | `ems-upload` alone |
| `LISTING_HTTP_PORT` | 8086 | listing |
| `STREAMING_HTTP_PORT` | 8087 | streaming |
| `PROCESSOR_HTTP_PORT` | 8088 | IMS processor (first worker; extras are +1, +2, …) |

## Required / common

| Var | Default / notes |
|---|---|
| `DATABASE_URL` | **required** — `postgres://streaming:streaming@localhost:5432/streaming` |
| `REDIS_URL` | `redis://localhost:6379` |
| `KAFKA_BOOTSTRAP_SERVERS` | `localhost:9092` |
| `KAFKA_TOPIC` | `video.uploaded` |
| `KAFKA_GROUP_ID` | `ims-processor` |
| `AWS_ENDPOINT_URL` | `http://localhost:4566` |
| `S3_PUBLIC_ENDPOINT` | same as AWS endpoint if unset — must be host-reachable for PUT |
| `S3_BUCKET` | `videos` |
| `AWS_ACCESS_KEY_ID` / `SECRET` | `test` / `test` |
| `AWS_DEFAULT_REGION` | `us-east-1` |
| `MAX_UPLOAD_BYTES` | `5368709120` (5 GiB) |
| `ALLOWED_CONTENT_TYPES` | mp4/webm/quicktime/matroska |
| `MAX_TITLE_CHARS` | `200` |
| `UPLOAD_PART_SIZE_BYTES` | `16777216` |
| `PRESIGN_TTL_SECS` | `3600` |
| `SESSION_TTL_SECS` | `86400` |
| `WATCH_TTL_SECS` | `2592000` (30 days) — streamer resume keys |
| `CDN_BASE_URL` | `http://localhost:8081` |
| `HLS_SEGMENT_SECS` | `6` — IMS FFmpeg HLS segment length |
| `IMS_CHUNK_SECS` | `60` — long-file time slice for parallel FFmpeg (`duration > 1.5 × chunk`) |
| `IMS_ENCODE_PARALLEL` | `0` — max concurrent FFmpeg jobs (`0` = CPU count clamped 4–16). Helm local: `2` (packed ladder is heavier per job) |
| `FFMPEG_PRESET` | `veryfast` (Helm local: `ultrafast`) |
| `IMS_WORKERS` | `3` — extra IMS processes from `scripts/dev.sh` |
| `FFMPEG_PATH` | `ffmpeg` |
| `IMS_WORK_DIR` | `/tmp/ims-processor` (dev.sh suffixes `-{port}` per worker) |
| `RUST_LOG` | see template (debug for app crates) |
| `LOG_DIR` | `logs` (export.sh → `<repo>/logs`) — file logs beside stdout |
| `LOG_NAME` | optional override for `{LOG_DIR}/{name}.log` (default: binary name) |

## Infra host ports (Compose)

| Port | Service |
|---|---|
| 4566 | LocalStack |
| 5432 | Postgres |
| 6379 | Redis |
| 9092 | Kafka (host) |
| 8081 | nginx CDN |
| 8082 | pgweb |
| 8083 | Redis Commander |
| 8084 | Kafka UI |
| 9008 | LocalStack UI (via nginx) |
