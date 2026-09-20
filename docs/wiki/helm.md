# Helm — full stack on Kubernetes

Chart: `helm/streaming/` · local overlay: `values-local.yaml`

Deploys **Postgres, Redis, Kafka, MinIO S3, nginx CDN, EMS, IMS, frontend, pgweb**.

## Local (Docker Desktop Kubernetes)

```bash
./scripts/k8s-up.sh
```

The frontend Service is **LoadBalancer** (`values-local.yaml` sets `service.frontendType`) —
Docker Desktop's Kubernetes auto-exposes it on **http://127.0.0.1:5173**, no port-forward or
terminal to keep open. The other services (EMS, MinIO, CDN, pgweb) are still
ClusterIP, so keep a port-forward running for those:

```bash
./scripts/k8s-pf.sh
```

Live IMS/EMS logs (stdout; repo `logs/*.log` is host `dev.sh` only):

```bash
kubectl -n streaming logs -l app.kubernetes.io/component=ims -f --prefix
kubectl -n streaming logs -l app.kubernetes.io/component=ems -f
```

| URL | Service |
|---|---|
| http://127.0.0.1:5173 | Uploader UI |
| http://127.0.0.1:8080 | EMS |
| http://127.0.0.1:9000 | MinIO S3 API (browser PUT) |
| http://127.0.0.1:8081 | CDN |
| http://127.0.0.1:8082 | pgweb (Postgres) |
| http://127.0.0.1:9001 | MinIO console (`minioadmin` / `minioadmin`) |

```bash
helm uninstall streaming -n streaming
```

Laptop defaults: 1 EMS, 3 IMS, HPA off, `imagePullPolicy: Never`, IMS `preset: ultrafast`, `chunkSecs: 60`, `encodeParallel: 2`, CPU limit `4`, IMS Deployment **Recreate** (Helm `--server-side=false` when switching from RollingUpdate). Docker Desktop Kubernetes runs in a **kind** node — `k8s-up.sh` imports local images into `desktop-control-plane`. Long files use a **packed ABR** encode (one FFmpeg per time slice; rungs taller than the source are skipped).

## Images (from source)

```bash
docker build -f backend/Dockerfile -t ems:latest backend
docker build -f backend/Dockerfile.ims -t ims-processor:latest backend
docker build -f frontend/Dockerfile -t frontend:latest frontend
```

## Templates

| Path | Role |
|---|---|
| `templates/postgres.yaml` · `redis.yaml` · `kafka.yaml` | Data plane |
| `templates/kafka-init.yaml` | `video.uploaded` × 12 partitions |
| `templates/minio.yaml` | S3 + console (`:9001`); `bucket-init` sidecar creates `videos` and opens `hls/` for anonymous read. On by default (`minio.enabled`) |
| `templates/localstack.yaml` · `localstack-ui.yaml` | LocalStack S3 + StackPort — **disabled** (`localstack.enabled: false`); enable one S3 backend at a time |
| `templates/pgweb.yaml` | Postgres browser UI (`:8082`) |
| `templates/nginx.yaml` | CDN `/videos/` |
| `templates/ems.yaml` · `ims.yaml` | App + IMS HPA (optional) |
| `templates/frontend.yaml` | SPA + API proxy (`/uploader`, `/lister`, `/streamer`) |
| `templates/configmap.yaml` | In-cluster env |
