# EMS listing & streaming

## Listing

| Item | Value |
|---|---|
| Lib | `backend/ems/listing/src/lib.rs` |
| Route | `GET /lister/videos` |
| Status | **Stub** `501` |
| Standalone port | `LISTING_HTTP_PORT` default **8086** |

## Streaming

| Item | Value |
|---|---|
| Lib | `backend/ems/streaming/src/` |
| Route | `GET /streamer/stream?file_id=` |
| Status | **Done** — returns CDN master playlist when `ready` |
| Other | `POST /streamer/save-user-state` still `501` |
| Standalone port | `STREAMING_HTTP_PORT` default **8087** |

### Response

```json
{
  "file_id": "…",
  "status": "ready",
  "master_playlist_url": "http://localhost:8081/videos/hls/{file_id}/master.m3u8"
}
```

Errors: `404` unknown / deleted · `409` not ready / no `playback_path`.

### Layout

| Path | Role |
|---|---|
| `app/mod.rs` | `GetStream` use case |
| `adapters/mod.rs` | Postgres `get_for_stream` (PK lookup) |
| `api/routes.rs` | HTTP |

Playback bytes: nginx → LocalStack S3 (not this service).
