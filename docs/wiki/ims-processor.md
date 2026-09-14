# IMS processor (stub)

| Item | Value |
|---|---|
| Lib | `backend/ims/processor/src/lib.rs` |
| Bin | `backend/ims/processor/src/bin/ims-processor.rs` |
| Today | `GET /health` only |
| Port | `PROCESSOR_HTTP_PORT` default **8088** |

## Planned behavior

1. Consume Kafka `video.uploaded` (`file_id`, `object_key`)
2. Read `s3://videos/raw/{file_id}/source`
3. Transcode + HLS → `s3://videos/hls/{file_id}/…`
4. Set Postgres status `processing` → `ready` (or `failed`)
5. Optionally invalidate listing cache

Not started — do not assume FFmpeg or consumer code exists yet.
