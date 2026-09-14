# Status

| Area | State | Notes |
|---|---|---|
| Docker data plane | Done | LocalStack, Postgres, Redis, Kafka, nginx + UIs |
| EMS gateway `ems` | Done | One process, one domain (`:8080`) |
| Upload controller | Done | Limits, abort, size verify, event_published, CORS, tests |
| Listing | Stub | `GET /lister/videos` → `501` |
| Streaming | Stub | `/streamer/*` → `501` |
| IMS processor | Stub | Health only; no Kafka consume yet |
| Frontend | Empty | `frontend/.gitkeep` |

Suggested next slice: IMS consume `video.uploaded` → HLS → implement listing + stream URL.
