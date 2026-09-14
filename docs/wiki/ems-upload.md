# EMS upload

Implemented. Used by gateway and standalone `ems-upload`.

## Key files

| Concern | Path |
|---|---|
| Composition / `build_state` | `backend/ems/upload/src/lib.rs` |
| Config | `backend/ems/upload/src/config.rs` |
| Routes + `AppState` | `backend/ems/upload/src/api/routes.rs` |
| DTOs | `backend/ems/upload/src/api/dto.rs` |
| Errors | `backend/ems/upload/src/api/error.rs` |
| Use case: get URL | `backend/ems/upload/src/app/get_upload_url.rs` |
| Use case: complete | `backend/ems/upload/src/app/complete_upload.rs` |
| Domain video / session | `backend/ems/upload/src/domain/` |
| Ports | `backend/ems/upload/src/ports/` |
| Postgres | `backend/ems/upload/src/adapters/postgres.rs` |
| Redis | `backend/ems/upload/src/adapters/redis.rs` |
| S3 | `backend/ems/upload/src/adapters/s3/` (`mod`, `presign`, `multipart`, `head`) |
| Kafka | `backend/ems/upload/src/adapters/kafka.rs` |
| Migrations | `backend/ems/upload/migrations/` |

## HTTP API

| Method | Path | Body | Result |
|---|---|---|---|
| `POST` | `/uploader/get-upload-url` | `{ file_size, title?, content_type? }` | `file_id`, `object_key`, `mode`, `parts[]` |
| `POST` | `/uploader/upload-completed` | `{ file_id }` | `{ file_id, status, object_key }` |

Multipart when `file_size > UPLOAD_PART_SIZE_BYTES` (default 16 MiB).  
Complete: HeadObject (+ complete multipart if needed) → `uploaded` → Kafka `video.uploaded` (idempotent if already past pending).

## Object key

`raw/{file_id}/source` (see `domain/video.rs`).

## Dual S3 endpoints

| Env | Role |
|---|---|
| `AWS_ENDPOINT_URL` | SDK / HeadObject (Docker-reachable) |
| `S3_PUBLIC_ENDPOINT` | Host embedded in presigned URLs (browser/curl) |

## Standalone

`cargo run -p ems-upload --bin ems-upload` → `HTTP_PORT` (8085) includes `/health` `/ready`.
