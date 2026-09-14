# IMS processor

Kafka worker: `video.uploaded` → claim DB row → download raw → **FFmpeg ABR HLS** → upload `hls/{id}/` → `ready`.

| Item | Value |
|---|---|
| Lib | `backend/ims/processor/src/` |
| Bin | `ims-processor` · health `:PROCESSOR_HTTP_PORT` (8088) |
| Group | `KAFKA_GROUP_ID` default `ims-processor` |
| Requires | `ffmpeg` on `PATH` (or `FFMPEG_PATH`) |

## Flow

1. Consume Kafka `VideoUploaded { file_id, object_key }` (key = `file_id`)
2. `UPDATE` claim: `uploaded|processing` → `processing` (indexed)
3. Download `object_key` from S3 to `IMS_WORK_DIR`
4. FFmpeg **ABR ladder** (`HLS_SEGMENT_SECS`, default 6): **360p / 720p / 1080p** — **one process per rung in parallel**, then write `master.m3u8`
5. Upload tree:
   - `hls/{file_id}/master.m3u8`
   - `hls/{file_id}/v0|v1|v2/index.m3u8` + `seg_*.ts`
6. Set `status=ready`, `playback_path=hls/{file_id}/master.m3u8`
7. Update `video_pipeline_steps`: queue=done, process=running→done, ready=done

On failure: `status=failed` + process step `failed`. Duplicate events for `ready` rows are skipped.  
Audio-less sources retry as video-only variants (all rungs).

## Layout

| Path | Role |
|---|---|
| `app/process_uploaded.rs` | Use case + pipeline step updates |
| `adapters/kafka_consumer.rs` | Consumer loop |
| `adapters/pipeline.rs` | Writes `video_pipeline_steps` |
| `adapters/ffmpeg.rs` · `ffmpeg_args.rs` | Parallel per-rung ABR HLS transcoder |
| `adapters/master_playlist.rs` | Writes multi-variant `master.m3u8` |
| `adapters/ladder.rs` | 360p / 720p / 1080p rungs |
| `adapters/hls_files.rs` | Collect + relative keys |
| `adapters/media_sniff.rs` | Magic: ISO BMFF (`ftyp`/`mdat`/…) then EBML; ZIP last |
| `adapters/s3.rs` / `postgres.rs` | IO |
| `ports/` | Traits |

## Env

See [env-and-ports.md](env-and-ports.md): `HLS_SEGMENT_SECS`, `FFMPEG_PATH`, `IMS_WORK_DIR`, `KAFKA_*`.

CDN play: `{CDN_BASE_URL}/videos/hls/{file_id}/master.m3u8` via [ems-stubs.md](ems-stubs.md) streaming API.
