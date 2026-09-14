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
| Listing / streaming stubs                | [ems-stubs.md](ems-stubs.md)         |
| IMS processor stub                       | [ims-processor.md](ims-processor.md) |
| Postgres schema & indexes                | [data-model.md](data-model.md)       |
| Env vars & ports                         | [env-and-ports.md](env-and-ports.md) |
| Docker / LocalStack / nginx              | [infra.md](infra.md)                 |
| Coding conventions (SOLID, 200 lines, …) | [conventions.md](conventions.md)     |
| Commit message linting ([commitlint](https://commitlint.js.org/)) | [../commitlint.md](../commitlint.md) |

## Route by path prefix

| Path                                  | Wiki                                 |
| ------------------------------------- | ------------------------------------ |
| `backend/ems/server/`                 | [ems-server.md](ems-server.md)       |
| `backend/ems/upload/`                 | [ems-upload.md](ems-upload.md)       |
| `backend/ems/listing/` · `streaming/` | [ems-stubs.md](ems-stubs.md)         |
| `backend/ims/processor/`              | [ims-processor.md](ims-processor.md) |
| `backend/shared/`                     | [architecture.md](architecture.md)   |
| `backend/ems/upload/migrations/`      | [data-model.md](data-model.md)       |
| `docker/`                             | [infra.md](infra.md)                 |
| `docs/local-setup/`                   | [env-and-ports.md](env-and-ports.md) |
| `.cursor/skills/`                     | [conventions.md](conventions.md)     |
| `LICENSE`                             | MIT — root [LICENSE](../../LICENSE)  |

## Agent habit

`INDEX → topic page → listed files`. Update wiki when structure changes.
