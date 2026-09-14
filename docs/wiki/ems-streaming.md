# EMS streaming

Implemented. Used by the gateway and standalone `ems-streaming`.

Playback **bytes** are not served here — nginx CDN → LocalStack S3.

## REST surface

| Method | Path | Action |
|---|---|---|
| `GET` | `/streamer/stream?file_id=` | CDN master playlist URL when `ready`; marks pipeline `play` done |
| `POST` | `/streamer/save-user-state` | Upsert watch progress (Redis, TTL) |
| `GET` | `/streamer/user-state?file_id=&viewer_id=` | Load last `position_secs` |

### Stream

```json
{
  "file_id": "…",
  "status": "ready",
  "master_playlist_url": "http://localhost:8081/videos/hls/{file_id}/master.m3u8"
}
```

Errors: `404` unknown / deleted · `409` not ready / no `playback_path`.

### Watch progress

`viewer_id` is a client UUID (desk stores it in `localStorage`). Redis key `watch:{viewer_id}:{file_id}`, TTL `WATCH_TTL_SECS` (default 30 days).

```json
{
  "file_id": "…",
  "viewer_id": "…",
  "position_secs": 12.5,
  "duration_secs": 120.0
}
```

`POST` rejects non-finite / out-of-range seconds (`400`). `GET` returns `404` when nothing is stored. `position_secs` / `duration_secs` max **86400**.

## Key files

| Concern | Path |
|---|---|
| Stream use case | `backend/ems/streaming/src/app/get_stream.rs` |
| Watch use cases | `app/save_user_state.rs` · `app/get_user_state.rs` |
| Ports | `ports/mod.rs` (`VideoRepository`, `WatchProgressStore`) |
| Adapters | `adapters/postgres.rs` · `adapters/redis.rs` · `adapters/pipeline.rs` |
| HTTP | `api/routes.rs` · `api/watch.rs` · `api/dto.rs` |
| Composition | `lib.rs` (`build_router`) |

## Standalone

`cargo run -p ems-streaming --bin ems-streaming` → `STREAMING_HTTP_PORT` (8087).  
Tests: `cargo test -p ems-streaming`.
