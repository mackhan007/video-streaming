# Status

| Area | State | Notes |
|---|---|---|
| Docker data plane | Done | LocalStack, Postgres, Redis, Kafka, nginx + UIs |
| EMS gateway `ems` | Done | One process, one domain (`:8080`) |
| Upload controller | Done | Limits, abort, size verify, event_published, CORS, tests |
| Listing | Stub | `GET /lister/videos` → `501` |
| Streaming | **Done** | `GET /streamer/stream?file_id=` → CDN master URL |
| IMS processor | **Done** | Kafka → chunked parallel FFmpeg HLS → S3 → `ready` + pipeline steps |
| Helm / Kubernetes | **Done** | Full stack chart + `scripts/k8s-up.sh` ([helm.md](helm.md)) |
| Pipeline tracking | **Done** | `video_pipeline_steps` table + `GET …/pipeline` UI |
| Frontend | **Uploader done** | React Vite desk — [frontend.md](frontend.md) |

Suggested next slice: listing catalog + `save-user-state` + player UI.
