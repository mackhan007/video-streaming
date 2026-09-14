# IMS processor

Kafka worker: `video.uploaded` → claim DB row → download raw → **FFmpeg ABR HLS** → upload `hls/{id}/` → `ready`.

| Item | Value |
|---|---|
| Lib | `backend/ims/processor/src/` |
| Bin | `ims-processor` · health `:PROCESSOR_HTTP_PORT` (8088) |
| Group | `KAFKA_GROUP_ID` default `ims-processor` (one consumer per replica / `IMS_WORKERS`) |
| Requires | `ffmpeg` + `ffprobe` on `PATH` (or `FFMPEG_PATH`) |
| Scale | Kafka topic partitions ≥ workers ([infra.md](infra.md) `kafka-init` = 12). Long files enqueue per-chunk jobs; **every IMS replica** claims work (`SKIP LOCKED`). Each pod downloads the source **once** (`SourceCache` + `source.name`) so parallel FFmpeg jobs do not clobber the file. |

## Flow

1. Consume Kafka `VideoUploaded { file_id, object_key }` (key = `file_id`)
2. `UPDATE` claim: `uploaded|processing` → `processing` (indexed)
3. Download `object_key` from S3 to `IMS_WORK_DIR`
4. FFmpeg **ABR ladder** (`HLS_SEGMENT_SECS`, default 6): **360p / 720p / 1080p**. Short files: one FFmpeg per rung on the Kafka consumer. Long files (`duration > 1.5 × IMS_CHUNK_SECS`): insert `ims_encode_jobs` (**one row per chunk**; `pack_ladder` — one FFmpeg decodes once and writes all rungs). **All IMS pods** claim jobs (`FOR UPDATE SKIP LOCKED`), encode, upload `hls/{id}/v{r}/cNN/`, then one replica merges playlists + `master.m3u8`. Logs: `packed ladder jobs shared across IMS replicas`, `claimed encode job` (`pack=true`), `chunk ladder encode start` / `chunk encode ok`. `chunked abr encode` is only the **in-process** fallback path. Startup converts leftover per-rung batches (`upgraded encode batch to packed ladder`), **pauses encode batches whose video is not `processing`**, and requeues `running` jobs left by dead pods. Claim order: newest encoding batch, then `chunk_index` (so the file you just uploaded is not starved by older jobs). IMS uses Deployment `Recreate` so those startup repairs are safe.
5. Upload tree:
   - `hls/{file_id}/master.m3u8`
   - `hls/{file_id}/v0|v1|v2/index.m3u8` + `seg_*.ts` (chunked: `cNN/seg_*.ts`)
6. Set `status=ready`, `playback_path=hls/{file_id}/master.m3u8`
7. Update `video_pipeline_steps`: queue=done, process=running→done, ready=done

On failure: `status=failed` + process step `failed`. Duplicate events for `ready` rows are skipped.  
`POST /uploader/videos/{id}/retry` re-queues `failed` / `processing` / `uploaded` (not `pending`/`ready`). IMS **revives** a failed `ims_encode_batches` row (jobs `failed|running` → `queued`) instead of skipping because the batch already exists.

## Layout

| Path | Role |
|---|---|
| `app/process_uploaded.rs` | Use case + pipeline step updates |
| `app/enqueue_chunks.rs` | Probe duration → insert chunk jobs |
| `app/job_loop.rs` · `run_encode_job.rs` | Replica workers + HLS finalize |
| `app/ensure_source.rs` | Per-file source download gate |
| `adapters/postgres_jobs/` | SKIP LOCKED claim / enqueue |
| `adapters/kafka_consumer.rs` | Consumer loop |
| `adapters/pipeline.rs` | Writes `video_pipeline_steps` |
| `adapters/ffmpeg.rs` · `ffmpeg_args.rs` | ABR HLS transcoder (`FfmpegHls`) |
| `adapters/ffmpeg_abr.rs` | Full-file vs chunked ABR + audio fallback |
| `adapters/ffmpeg_chunked.rs` | Chunk jobs behind a semaphore (packed ladder locally) |
| `adapters/ffmpeg_chunk_ladder.rs` | One FFmpeg: `split` + all ABR HLS outputs |
| `adapters/ffmpeg_plan.rs` · `ffmpeg_probe.rs` · `ffmpeg_merge.rs` | Duration probe, chunk plan, playlist merge |
| `adapters/master_playlist.rs` | Writes multi-variant `master.m3u8` |
| `adapters/ladder.rs` | 360p / 720p / 1080p rungs |
| `adapters/hls_files.rs` | Collect + relative keys |
| `adapters/media_sniff.rs` | Magic: ISO BMFF (`ftyp`/`mdat`/…) then EBML; ZIP last |
| `adapters/s3.rs` / `postgres.rs` | IO |
| `ports/` | Traits |

## Env

See [env-and-ports.md](env-and-ports.md): `HLS_SEGMENT_SECS`, `IMS_CHUNK_SECS`, `IMS_ENCODE_PARALLEL`, `FFMPEG_PRESET`, `FFMPEG_PATH`, `IMS_WORK_DIR`, `IMS_WORKERS`, `KAFKA_*`.

K8s: [helm.md](helm.md).

CDN play: `{CDN_BASE_URL}/videos/hls/{file_id}/master.m3u8` via [ems-stubs.md](ems-stubs.md) streaming API.
