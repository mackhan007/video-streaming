# Status

| Area | State | Notes |
|---|---|---|
| Docker data plane | Done | LocalStack, Postgres, Redis, Kafka, nginx + UIs |
| EMS gateway `ems` | Done | One process, one domain (`:8080`) |
| Upload controller | Done | Limits, abort, size verify, event_published, CORS, tests |
| Listing | **Done** | `GET /lister/videos` (uploads) · `GET /lister/links` (ready URLs) |
| Streaming | **Done** | `GET /streamer/stream` + watch progress (`save-user-state` / `user-state`) |
| IMS processor | **Done** | Kafka → chunked parallel FFmpeg HLS → S3 → `ready` + pipeline steps |
| Helm / Kubernetes | **Done** | Full stack chart + `scripts/k8s-up.sh` ([helm.md](helm.md)) |
| Pipeline tracking | **Done** | `video_pipeline_steps` table + `GET …/pipeline` UI |
| Frontend | **Done** | Upload + My videos + Watch — [frontend.md](frontend.md) |

Suggested next slice: richer player UX (quality picker) if needed.
