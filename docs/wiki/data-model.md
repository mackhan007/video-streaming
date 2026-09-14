# Data model

## Migrations

Path: `backend/ems/upload/migrations/`  
Applied automatically via sqlx in `PostgresVideoRepository::migrate` on EMS/upload start.

| File | Contents |
|---|---|
| `20240910000001_create_videos.sql` | enum `video_status`, table `videos`, index `(status, created_at DESC)` |
| `20240910000002_videos_hot_path_indexes.sql` | ready partial, `(status, updated_at)`, unique `object_key`, partial `playback_path` |
| `20240914000003_event_published.sql` | `event_published` for Kafka at-least-once |
| `20240914000004_soft_delete.sql` | `deleted_at` + partial index for live rows |

## `video_status`

`pending` → `uploaded` → `processing` → `ready` (or `failed`)

## `videos` columns (summary)

`id` (UUID PK), `status`, `title`, `content_type`, `file_size`, `object_key`, `upload_id`, `part_size`, `playback_path`, `created_at`, `updated_at`

## Indexes (hot path)

| Index | Purpose |
|---|---|
| PK `id` | Point get / update |
| `videos_status_created_at` | Status-scoped lists |
| `videos_ready_created_at` | Listing ready feed |
| `videos_status_updated_at` | IMS queues |
| `videos_object_key_uidx` | Unique storage key |
| `videos_playback_path` | Resolve CDN path |

## Redis

Key `upload:{file_id}` — JSON `UploadSession`, TTL `SESSION_TTL_SECS`.

## Kafka

Topic `KAFKA_TOPIC` (default `video.uploaded`). Payload: `VideoUploaded { file_id, object_key }`. Key: `file_id`.
