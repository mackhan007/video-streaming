# Env and ports

Templates: `docs/local-setup/.env.template` · loader: `docs/local-setup/export.sh`  
Config parse: `backend/ems/upload/src/config.rs`

## App ports

| Var | Default | Who |
|---|---|---|
| `EMS_HTTP_PORT` | 8080 | `ems` gateway |
| `HTTP_PORT` | 8085 | `ems-upload` alone |
| `LISTING_HTTP_PORT` | 8086 | listing stub |
| `STREAMING_HTTP_PORT` | 8087 | streaming stub |
| `PROCESSOR_HTTP_PORT` | 8088 | IMS stub |

## Required / common

| Var | Default / notes |
|---|---|
| `DATABASE_URL` | **required** — `postgres://streaming:streaming@localhost:5432/streaming` |
| `REDIS_URL` | `redis://localhost:6379` |
| `KAFKA_BOOTSTRAP_SERVERS` | `localhost:9092` |
| `KAFKA_TOPIC` | `video.uploaded` |
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
| `CDN_BASE_URL` | `http://localhost:8081` |
| `RUST_LOG` | see template (debug for app crates) |

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
