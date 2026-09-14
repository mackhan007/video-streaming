# Infra (Docker)

Compose file: `docker/docker-compose.yml`

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Services → code relevance

| Compose service | Host | Used by |
|---|---|---|
| `localstack` | 4566 | S3 adapter; bucket init script |
| `postgres` | 5432 | Video repository |
| `redis` | 6379 | Upload sessions |
| `kafka` | 9092 / 29092 | Event publisher |
| `nginx` | 8081, 9008 | CDN + StackPort proxy |

## Init / config files

| File | Role |
|---|---|
| `docker/localstack/init/ready.d/01-create-videos-bucket.sh` | Create `videos` + CORS |
| `docker/nginx/nginx.conf` | `/videos/` → LocalStack; `:9008` UI |

## S3 key layout

```
s3://videos/raw/{file_id}/source
s3://videos/hls/{file_id}/master.m3u8
```

## EMS in Docker (optional)

`backend/Dockerfile` builds release `ems`. Prefer host `cargo run` for day-to-day.

Full runbook: [../local-setup.md](../local-setup.md).
