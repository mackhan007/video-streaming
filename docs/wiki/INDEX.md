# Code wiki — INDEX

**Start here.** Pick a topic → open that page → open only the paths it lists.

Human overview: [README.md](../../README.md) · Runbook: [local-setup.md](../local-setup.md)

## Route by question

| If you need…                             | Go to                                |
| ---------------------------------------- | ------------------------------------ |
| What exists / stubs vs done              | [status.md](status.md)               |
| Repo map, crates, bins                   | [architecture.md](architecture.md)   |
| Unified EMS (`ems`) routes & wiring      | [ems-server.md](ems-server.md)       |
| Upload flow, ports, adapters             | [ems-upload.md](ems-upload.md)       |
| Listing catalog (uploads + ready links)  | [ems-listing.md](ems-listing.md)     |
| Streaming (CDN playlist + watch progress) | [ems-streaming.md](ems-streaming.md) |
| IMS processor (FFmpeg / Kafka workers)   | [ims-processor.md](ims-processor.md) |
| Postgres schema & indexes                | [data-model.md](data-model.md)       |
| Env vars & ports                         | [env-and-ports.md](env-and-ports.md) |
| Docker / MinIO / nginx                   | [infra.md](infra.md)                 |
| Swapping the S3 backend (checklist)      | [s3-backend-migrations.md](s3-backend-migrations.md) |
| Helm / Kubernetes (full stack)           | [helm.md](helm.md)                   |
| Coding conventions (SOLID, 200 lines, …) | [conventions.md](conventions.md)     |
| **Commit past changes** (one commit per task) | [commit-past-changes.md](commit-past-changes.md) |
| Commit message linting ([commitlint](https://commitlint.js.org/)) | [../commitlint.md](../commitlint.md) |
| React desk UI (upload, library, watch)    | [frontend.md](frontend.md)           |
| Dev script (EMS + IMS workers + UI)      | [frontend.md](frontend.md) · `scripts/dev.sh` |

## Route by path prefix

| Path                                  | Wiki                                 |
| ------------------------------------- | ------------------------------------ |
| `backend/ems/server/`                 | [ems-server.md](ems-server.md)       |
| `backend/ems/upload/`                 | [ems-upload.md](ems-upload.md)       |
| `backend/ems/listing/`                | [ems-listing.md](ems-listing.md)     |
| `backend/ems/streaming/`              | [ems-streaming.md](ems-streaming.md) |
| `backend/ims/processor/`              | [ims-processor.md](ims-processor.md) |
| `backend/shared/`                     | [architecture.md](architecture.md)   |
| `backend/ems/upload/migrations/`      | [data-model.md](data-model.md)       |
| `docker/`                             | [infra.md](infra.md)                 |
| `helm/`                               | [helm.md](helm.md)                   |
| `docs/local-setup/`                   | [env-and-ports.md](env-and-ports.md) |
| `.cursor/skills/`                     | [conventions.md](conventions.md)     |
| `frontend/`                             | [frontend.md](frontend.md)           |
| `scripts/dev.sh`                        | [frontend.md](frontend.md)           |
| `scripts/k8s-up.sh` · `helm/`           | [helm.md](helm.md)                   |
| `LICENSE`                             | MIT — root [LICENSE](../../LICENSE)  |

## Agent habit

`INDEX → topic page → listed files`. Update wiki when structure changes.
