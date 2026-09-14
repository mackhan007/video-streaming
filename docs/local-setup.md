# Local setup

Docker Compose runs the local data plane: LocalStack (S3), Postgres, Redis, Kafka, and nginx as the local CDN. The EMS API runs on the **host** with the system Rust toolchain.

Architecture and API design: [README.md](../README.md).  
Code map (start here for file routing): [wiki/INDEX.md](wiki/INDEX.md).

## Start infra

```bash
docker compose -f docker/docker-compose.yml up -d
docker compose -f docker/docker-compose.yml ps
```

Stop (keeps volumes): `docker compose -f docker/docker-compose.yml down`  
Clean slate: add `-v`.

Volumes: `localstack_data`, `postgres_data`, `redis_data`, `kafka_data`, `nginx_cache`.

## Browser UIs

Do **not** open raw API ports (4566, 5432, 6379, 9092) in a browser.

| What | URL | Notes |
|---|---|---|
| LocalStack (StackPort) | http://localhost:9008 | Proxied to `localstack:4566` |
| Postgres (pgweb) | http://localhost:8082 | DB `streaming` |
| Redis Commander | http://localhost:8083 | |
| Kafka UI | http://localhost:8084 | `video.uploaded` after first publish |
| nginx CDN | http://localhost:8081 | `/videos/…` → LocalStack S3 |
| **Macky Uploader** | http://localhost:5173 | React desk → EMS `:8080` (Vite proxy) |

## Host ↔ container endpoints

| Service | From host | From Compose network |
|---|---|---|
| LocalStack S3 | `http://localhost:4566` | `http://localstack:4566` |
| Postgres | `localhost:5432` | `postgres:5432` |
| Redis | `localhost:6379` | `redis:6379` |
| Kafka | `localhost:9092` | `kafka:29092` |
| nginx CDN | `http://localhost:8081` | `http://nginx` |
| **EMS API** | `http://localhost:8080` | (runs on host by default) |

### LocalStack

| Setting | Value |
|---|---|
| Region | `us-east-1` |
| Credentials | `test` / `test` |
| Bucket | `videos` (created on first start + CORS) |
| Addressing | Path-style |

```bash
export AWS_CONFIG_FILE="$PWD/docs/local-setup/config"
export AWS_SHARED_CREDENTIALS_FILE="$PWD/docs/local-setup/credentials"
aws s3 ls --profile localstack
aws s3 ls s3://videos --recursive --profile localstack
# or: docker exec localstack awslocal s3 ls s3://videos --recursive
```

Keys:

```
s3://videos/raw/{file_id}/source
s3://videos/hls/{file_id}/master.m3u8
```

### Postgres

`postgres://streaming:streaming@localhost:5432/streaming`  
Migrations (videos + indexes) apply automatically when EMS/upload starts.

### Redis / Kafka

Redis: no password (`localhost:6379`).  
Kafka topic `video.uploaded` is created/altered to **12 partitions** by compose `kafka-init` (`localhost:9092`) so several IMS consumers can share work.

## Env

```bash
cp docs/local-setup/.env.template .env
./docs/local-setup/export.sh    # new shell with env loaded
# or: source docs/local-setup/export.sh
```

Important vars: `DATABASE_URL`, `REDIS_URL`, `KAFKA_BOOTSTRAP_SERVERS`, `KAFKA_TOPIC`, `AWS_ENDPOINT_URL`, `S3_PUBLIC_ENDPOINT`, `S3_BUCKET`, `EMS_HTTP_PORT`, `RUST_LOG`, `CDN_BASE_URL`.

Presigned browser/curl uploads must use **`S3_PUBLIC_ENDPOINT=http://localhost:4566`**. In-container workers would use `localstack:4566` for the SDK endpoint only.

## Uploader UI

With infra up:

```bash
npm run dev          # EMS :8080 + IMS × IMS_WORKERS (default 3 on 8088+) + Vite :5173
```

Requires **ffmpeg** on `PATH` (or `FFMPEG_PATH`) for HLS chunking.

Or only the UI (EMS + IMS already running):

```bash
cd frontend && npm install && npm run dev
```

Open http://localhost:5173. Details: [wiki/frontend.md](wiki/frontend.md).

## Kubernetes (Docker Desktop)

Enable Kubernetes in Docker Desktop, then:

```bash
./scripts/k8s-up.sh
```

Uploader: http://127.0.0.1:5173 · pgweb: http://127.0.0.1:8082 · StackPort: http://127.0.0.1:9008 — [wiki/helm.md](wiki/helm.md).

## Run EMS (system cargo)

Needs [rustup](https://rustup.rs) (`cargo` on `PATH`). On macOS install **cmake** for `rdkafka`: `brew install cmake`.  
Do **not** use a project-local `.cargo-cache`.

```bash
cd backend && cargo run -p ems-server --bin ems
# separate terminal — IMS worker
cd backend && cargo run -p ims-processor --bin ims-processor
```

| Path | Behavior |
|---|---|
| `GET /health` | liveness |
| `GET /ready` | Postgres + Redis + S3 |
| `POST /uploader/videos` | live — **201** create |
| `POST /uploader/videos/{id}/complete` | live |
| `POST /uploader/videos/{id}/abort` | live |
| `DELETE /uploader/videos/{id}` | live (soft delete) |
| `GET /lister/videos` | live — all uploads (keyset `limit`/`seen`) |
| `GET /lister/links` | live — ready videos + CDN playlist URLs |
| `GET /streamer/stream?file_id=` | live — CDN master URL when `ready` |
| `POST /streamer/save-user-state` | live — watch progress (Redis) |
| `GET /streamer/user-state?file_id=&viewer_id=` | live — last position |

Default port: **`EMS_HTTP_PORT=8080`**. Logging via `RUST_LOG` (see `.env.template`).

### Smoke test

```bash
curl -s localhost:8080/health
curl -si -X POST localhost:8080/uploader/videos \
  -H 'content-type: application/json' \
  -d '{"file_size":1024,"content_type":"video/mp4"}'
# PUT each returned parts[].url to LocalStack, then:
curl -s -X POST localhost:8080/uploader/videos/<uuid>/complete
# soft-delete example:
# curl -s -X DELETE localhost:8080/uploader/videos/<uuid>
```

### Standalone bins (optional)

| Bin | Port env | Default |
|---|---|---|
| `ems-upload` | `HTTP_PORT` | 8085 |
| `ems-listing` | `LISTING_HTTP_PORT` | 8086 |
| `ems-streaming` | `STREAMING_HTTP_PORT` | 8087 |
| `ims-processor` | `PROCESSOR_HTTP_PORT` | 8088 |

```bash
cargo run -p ems-upload --bin ems-upload
```

### Optional Docker image for EMS

```bash
docker build -f backend/Dockerfile -t ems backend
docker run --rm -p 8080:8080 --env-file .env \
  -e DATABASE_URL=postgres://streaming:streaming@host.docker.internal:5432/streaming \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  -e AWS_ENDPOINT_URL=http://host.docker.internal:4566 \
  -e S3_PUBLIC_ENDPOINT=http://localhost:4566 \
  -e KAFKA_BOOTSTRAP_SERVERS=host.docker.internal:9092 \
  ems
```

## Layout

```
backend/
  Cargo.toml                 # workspace; default-members = ems-server
  shared/                    # domain types + logging
  ems/server/                # bin ems — unified gateway
  ems/upload/                # upload lib + bin; migrations/
  ems/listing/               # catalog: /lister/videos + /lister/links
  ems/streaming/             # CDN playlist URLs + watch progress
  ims/processor/             # Kafka → chunked FFmpeg HLS
  Dockerfile                 # release image → ems :8080
  Dockerfile.ims             # release image → ims-processor
docker/
  docker-compose.yml
  kafka/create-topic.sh      # video.uploaded × 12 partitions
  localstack/init/ready.d/   # bucket videos + CORS
  nginx/nginx.conf           # :8081 CDN, :9008 StackPort
docs/local-setup.md
docs/wiki/                   # code map — start at INDEX.md
docs/local-setup/
  .env.template
  export.sh
  config / credentials       # AWS profile localstack
.cursor/skills/              # project agent skills
diagrams/
frontend/                    # React uploader (Vite :5173)
helm/streaming/              # K8s chart ([wiki/helm.md](wiki/helm.md))
scripts/k8s-up.sh            # build images + helm install + port-forward
```
