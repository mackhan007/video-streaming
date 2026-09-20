# Infra (Docker)

Compose file: `docker/docker-compose.yml`

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Services → code relevance

| Compose service | Host | Used by |
|---|---|---|
| `minio` | 9000 (S3), 9001 (console) | S3 adapter |
| `minio-init` | (oneshot) | `mc`: create `videos` bucket, anonymous-read on `hls/` |
| `postgres` | 5432 | Video repository |
| `redis` | 6379 | Upload sessions |
| `kafka` | 9092 / 29092 | Event publisher |
| `kafka-init` | (oneshot) | Create/alter `video.uploaded` to **12 partitions** |
| `nginx` | 8081 | CDN proxy → MinIO |

## Init / config files

| File | Role |
|---|---|
| `docker/docker-compose.yml` (`minio-init` service) | Create `videos` bucket + anonymous-read `hls/` (inline `mc` command) |
| `docker/nginx/nginx.conf` | `/videos/` → MinIO |
| `docker/kafka/create-topic.sh` | `video.uploaded` × 12 partitions (IMS consumer scale) |

K8s (Docker Desktop): [helm.md](helm.md) · `./scripts/k8s-up.sh` (includes pgweb `:8082` + MinIO console `:9001`). Extra local IMS processes on the host: `IMS_WORKERS` in [env-and-ports.md](env-and-ports.md).

LocalStack is kept as an alternate S3 backend behind a toggle (`helm/streaming/values.yaml`: `localstack.enabled` / `minio.enabled`, mutually exclusive) but isn't wired into `docker/docker-compose.yml` — MinIO is the only Compose backend. See [s3-backend-migrations.md](s3-backend-migrations.md) if you're swapping the backend again.

## S3 key layout

```
s3://videos/raw/{file_id}/source
s3://videos/hls/{file_id}/master.m3u8
```

## EMS in Docker (optional)

`backend/Dockerfile` builds release `ems`. Prefer host `cargo run` for day-to-day.

Full runbook: [../local-setup.md](../local-setup.md).
