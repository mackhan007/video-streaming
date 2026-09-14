# EMS listing & streaming (stubs)

Both return **`501 Not Implemented`** with a JSON hint. Mounted on the unified gateway.

## Listing

| Item | Value |
|---|---|
| Lib | `backend/ems/listing/src/lib.rs` |
| Route | `GET /lister/videos` |
| Planned query | `?limit=&seen=` |
| Standalone port | `LISTING_HTTP_PORT` default **8086** |

## Streaming

| Item | Value |
|---|---|
| Lib | `backend/ems/streaming/src/lib.rs` |
| Routes | `GET /streamer/stream`, `POST /streamer/save-user-state` |
| Planned | master playlist URL via `CDN_BASE_URL`; watch position body |
| Standalone port | `STREAMING_HTTP_PORT` default **8087** |

## When implementing

- Follow upload hexagonal layout (`api` / `app` / `ports` / `adapters`)
- Listing: use indexes in [data-model.md](data-model.md) (`videos_ready_created_at`, …)
- Playback bytes stay on nginx → S3, not the streaming service
