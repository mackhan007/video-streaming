# EMS upload

Implemented. Used by gateway and standalone `ems-upload`.

## REST surface

Resource: **`/uploader/videos`**

| Method   | Path                                  | Status               | Action                            |
| -------- | ------------------------------------- | -------------------- | --------------------------------- |
| `POST`   | `/uploader/videos`                    | **201** + `Location` | create upload / presign + init pipeline steps |
| `GET`    | `/uploader/videos/{file_id}`          | 200                  | video row status                  |
| `GET`    | `/uploader/videos/{file_id}/pipeline` | 200                  | steps + ABR `progress_pct` |
| `POST`   | `/uploader/videos/{file_id}/complete` | 200                  | verify S3 + mark uploaded + Kafka |
| `POST`   | `/uploader/videos/{file_id}/abort`    | 200                  | cancel pending                    |
| `POST`   | `/uploader/videos/{file_id}/retry`    | 200                  | re-queue failed/processing/uploaded → Kafka; IMS revives failed chunk jobs |
| `DELETE` | `/uploader/videos/{file_id}`          | 200                  | soft delete (`deleted_at`)        |

`file_id` is in the path (not the body) for complete / abort / delete.

## Key files

| Concern   | Path                                                                                     |
| --------- | ---------------------------------------------------------------------------------------- |
| Routes    | `backend/ems/upload/src/api/routes.rs`                                                   |
| Handlers  | `backend/ems/upload/src/api/handlers.rs`, `handlers_status.rs`, `handlers_pipeline.rs` |
| DTOs      | `backend/ems/upload/src/api/dto.rs`                                                      |
| Use cases | `app/get_upload_url.rs`, `complete_upload.rs`, `abort_upload.rs`, `get_pipeline.rs`, `pipeline_progress.rs`, … |
| Pipeline  | `ports/pipeline.rs` · `adapters/postgres/pipeline.rs` · `adapters/postgres/encode_progress.rs` |

## Limits (env)

| Var                      | Default                     |
| ------------------------ | --------------------------- |
| `MAX_UPLOAD_BYTES`       | 5 GiB                       |
| `ALLOWED_CONTENT_TYPES`  | mp4/webm/quicktime/matroska |
| `MAX_TITLE_CHARS`        | 200                         |
| `UPLOAD_PART_SIZE_BYTES` | 16 MiB (S3 max 10k parts)   |

## Dual S3 endpoints

| Env                  | Role               |
| -------------------- | ------------------ |
| `AWS_ENDPOINT_URL`   | SDK / HeadObject   |
| `S3_PUBLIC_ENDPOINT` | Presigned URL host |

## Standalone

`cargo run -p ems-upload --bin ems-upload` → `HTTP_PORT` (8085).  
Tests: `cargo test -p ems-upload`.
