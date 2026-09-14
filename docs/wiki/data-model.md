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
| `20240914000005_ims_claim_indexes.sql` | partial indexes for `uploaded` / `processing` IMS claims |
| `20240914000006_pipeline_steps.sql` | `video_pipeline_steps` + enums + `(video_id)` / `(state, updated_at)` indexes |
| `20240914000007_pipeline_abr_steps.sql` | enum values `hls_360` / `hls_720` / `hls_1080` |
| `20240914000008_ims_encode_jobs.sql` | `ims_encode_batches` / `ims_encode_jobs` + SKIP LOCKED claim indexes |
| `20240914000009_pack_ladder.sql` | `ims_encode_batches.pack_ladder` (one FFmpeg per chunk writes all rungs) |
| `20240914000010_claim_newest.sql` | queued `(video_id, chunk_index)` + encoding `created_at DESC` claim indexes |
| `20240914000011_encode_progress.sql` | `(video_id, rung, state)` for pipeline ABR % poll |

## `video_status`

`pending` → `uploaded` → `processing` → `ready` (or `failed`)

## Pipeline tracking (`video_pipeline_steps`)

Separate from `videos.status`. One row per `(video_id, step)`:

| step | Written by |
|---|---|
| `upload` | EMS create (running) → complete (done) / abort (failed) |
| `queue` | EMS complete (running→done after Kafka publish) |
| `process` | IMS Kafka worker (running → done / failed) |
| `hls_360` / `hls_720` / `hls_1080` | IMS during ABR encode (parallel FFmpeg per rung) |
| `ready` | IMS on success |
| `play` | Streaming when CDN URL is served |

States: `pending` \| `running` \| `done` \| `failed`.

API: `GET /uploader/videos/{file_id}/pipeline` (includes `started_at` / `finished_at`, plus `progress_pct` / `chunks_done` / `chunks_total` on process + ABR steps from `ims_encode_jobs`).

## `videos` columns (summary)

`id` (UUID PK), `status`, `title`, `content_type`, `file_size`, `object_key`, `upload_id`, `part_size`, `playback_path`, `event_published`, `deleted_at`, `created_at`, `updated_at`

## Indexes (hot path)

| Index | Purpose |
|---|---|
| PK `id` | Point get / update / stream lookup |
| `videos_status_created_at` | Status-scoped lists |
| `videos_ready_created_at` | Listing ready feed |
| `videos_status_updated_at` | IMS queues |
| `videos_uploaded_updated_at` | IMS claim uploaded rows |
| `videos_processing_updated_at` | Reclaim stuck processing |
| `videos_object_key_uidx` | Unique storage key |
| `videos_playback_path` | Resolve CDN path |

## Redis

Key `upload:{file_id}` — JSON `UploadSession`, TTL `SESSION_TTL_SECS`.

## Kafka

Topic `KAFKA_TOPIC` (default `video.uploaded`). Payload: `VideoUploaded { file_id, object_key }`. Key: `file_id`.

## Distributed IMS encode (`ims_encode_jobs`)

Long files: Kafka consumer **enqueues** one row per **chunk** (`pack_ladder`; one FFmpeg writes 360p/720p/1080p). All IMS replicas claim with `FOR UPDATE SKIP LOCKED` (`ims_encode_jobs_queued_video_chunk`, newest batch first). Only `videos.status = processing` is claimed. Startup heals orphaned `running` rows. Batch row `ims_encode_batches` (PK `video_id`) tracks finalize. Applied by EMS **and** IMS (`sqlx::migrate`).
