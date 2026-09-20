# S3 backend swaps — checklist

The S3 backend (MinIO / LocalStack / anything else S3-compatible) is referenced from far more
places than the Compose/Helm service definition. This came up migrating LocalStack → MinIO —
several of these were missed on the first pass. Work through the whole list, in both directions
(Compose **and** Helm), whenever the backend changes.

## Docker Compose (`docker/`)

| File | What to check |
| --- | --- |
| `docker/docker-compose.yml` | Service block itself, healthcheck, `depends_on` on `nginx`, the `volumes:` name at the bottom |
| `docker/nginx/nginx.conf` | `proxy_pass` targets under `location /videos/` and the `.m3u8` location; drop/add any backend-specific UI proxy block (e.g. LocalStack's StackPort had one) |
| Backend init (bucket + CORS/anonymous-read) | LocalStack used a mounted script (`docker/localstack/init/ready.d/`); MinIO uses an inline `mc` command in a `*-init` one-shot service — pick whichever fits the new backend, delete the old mechanism's directory if unused |

## Helm chart (`helm/streaming/`)

| File | What to check |
| --- | --- |
| `templates/<backend>.yaml` | Deployment/Service/init for the new backend; keep the old one guarded behind `{{- if .Values.<backend>.enabled }}` rather than deleting it, unless you're sure nobody needs the fallback |
| `templates/_helpers.tpl` | `streaming.s3Backend` / `streaming.s3Upstream` / `streaming.s3PublicEndpoint` — the mutually-exclusive enable/derive logic |
| `templates/configmap.yaml` | `AWS_ENDPOINT_URL` (uses the helper above — shouldn't need touching, but verify) |
| `templates/nginx.yaml` (CDN) | `proxy_pass` target under `location /videos/`; CORS header handling if the new backend answers CORS itself |
| `values.yaml` | `<backend>.enabled` toggle, `image.<backend>` (+ `image.mc`-equivalent if there's an init sidecar), `service.<backend>Port(s)`, `resources.<backend>` |
| `values-local.yaml` | Any local-only overrides (e.g. `service.minioType: LoadBalancer` so Docker Desktop exposes it on localhost without port-forward, mirroring `frontendType`) |
| `scripts/k8s-pf.sh` | Which Service name/port(s) it forwards — this repo's version already branches on `localstack.enabled`, extend that pattern rather than hardcoding one backend |

## Rust backend (`backend/`)

| File | What to check |
| --- | --- |
| `backend/ims/processor/src/config.rs` | Fallback defaults for `AWS_ENDPOINT_URL` / `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` when the env var is unset |
| `backend/ems/upload/src/config.rs` | Same three, plus `S3_PUBLIC_ENDPOINT`'s nested fallback |

These are just defaults (env vars always win), but a stale default is exactly the kind of thing
that works by accident in Compose/Helm (where the env is always set) and only bites someone
running a binary directly without a full `.env`.

## Local dev env files (`docs/local-setup/`)

| File | What to check |
| --- | --- |
| `.env.template` | `AWS_PROFILE`, `AWS_ENDPOINT_URL`, `AWS_ACCESS_KEY_ID`/`SECRET`, `S3_PUBLIC_ENDPOINT` |
| `credentials` | AWS CLI profile section name + keys |
| `config` | AWS CLI profile section name + `endpoint_url` |

## Docs

| File | What to check |
| --- | --- |
| `README.md` | Implementation-status table, request-path diagrams, the backend's own section (endpoint/console/credentials table), "why these pieces" table, repo layout tree |
| `docs/local-setup.md` | Volumes list, browser-UI table, host↔container endpoint table, the backend's own section + CLI example, env var list, repo layout tree |
| `docs/wiki/infra.md` | Compose service table, init/config file table |
| `docs/wiki/helm.md` | Access URL table, port-forward description, template table |
| `docs/wiki/env-and-ports.md` | `AWS_*` defaults, infra host-port table |

## Verify

After editing, grep for the old backend's name/port across the whole repo and confirm every hit
is either fixed or a deliberately-kept toggle/fallback (comment why):

```bash
grep -rIn "<old-backend-name>\|<old-port>" . \
  --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git
```

Then actually deploy and test both paths (`helm upgrade` + a real S3 PUT, `docker compose up` +
a real upload) — a clean grep doesn't prove the new backend works, only that the old one's name
isn't lying around.

**`helm upgrade` alone is not enough.** A ConfigMap change doesn't restart the pods that mount
it — `envFrom: configMapRef` env vars and files rendered into a volume are both fixed at pod
start. This bit us: the CDN `nginx` pod kept proxying to the old (now-deleted) backend's
ClusterIP and returned `504`s until it was explicitly restarted. After `helm upgrade`, run
`kubectl rollout restart` on every Deployment whose env or mounted config depends on the S3
backend — nginx (CDN), EMS, IMS — and verify with a real request through each one, not just
`kubectl get pods`.
