# Helm — full stack on Kubernetes

Chart: `helm/streaming/` · local overlay: `values-local.yaml`

Deploys **Postgres, Redis, Kafka, LocalStack S3, nginx CDN, EMS, IMS, frontend, pgweb, StackPort**.

## Local (Docker Desktop Kubernetes)

```bash
./scripts/k8s-up.sh
```

The frontend Service is **ClusterIP** (not on localhost by itself). Keep a port-forward running:

```bash
./scripts/k8s-pf.sh
```

Then open **http://127.0.0.1:5173** in Chrome/Safari (not the cluster IP `10.96.x.x`).

Live IMS/EMS logs (stdout; repo `logs/*.log` is host `dev.sh` only):

```bash
kubectl -n streaming logs -l app.kubernetes.io/component=ims -f --prefix
kubectl -n streaming logs -l app.kubernetes.io/component=ems -f
```

| URL | Service |
|---|---|
| http://127.0.0.1:5173 | Uploader UI |
| http://127.0.0.1:8080 | EMS |
| http://127.0.0.1:4566 | LocalStack S3 API (browser PUT) |
| http://127.0.0.1:8081 | CDN |
| http://127.0.0.1:8082 | pgweb (Postgres) |
| http://127.0.0.1:9008 | StackPort (S3 UI) |

```bash
helm uninstall streaming -n streaming
```

Laptop defaults: 1 EMS, 2 IMS, HPA off, `imagePullPolicy: Never`, IMS `preset: ultrafast`, `chunkSecs: 90`, `encodeParallel: 2`, CPU limit `4`, IMS Deployment **Recreate** (Helm `--server-side=false` when switching from RollingUpdate). Docker Desktop Kubernetes runs in a **kind** node — `k8s-up.sh` imports local images into `desktop-control-plane`. Long files use a **packed ABR** encode (one FFmpeg per time slice, all rungs).

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
| `templates/localstack.yaml` | S3 + CORS bucket `videos` |
| `templates/localstack-ui.yaml` | StackPort S3 UI (`:9008`) |
| `templates/pgweb.yaml` | Postgres browser UI (`:8082`) |
| `templates/nginx.yaml` | CDN `/videos/` |
| `templates/ems.yaml` · `ims.yaml` | App + IMS HPA (optional) |
| `templates/frontend.yaml` | SPA + API proxy |
| `templates/configmap.yaml` | In-cluster env |
