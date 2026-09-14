#!/usr/bin/env bash
# Build images and deploy the full stack to the current kubectl context.
# Usage (repo root): ./scripts/k8s-up.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="/opt/homebrew/bin:${HOME}/.cargo/bin:${PATH}"
NS="${K8S_NAMESPACE:-streaming}"
RELEASE="${HELM_RELEASE:-streaming}"
CHART="$ROOT/helm/streaming"

need() { command -v "$1" >/dev/null || { echo "missing $1" >&2; exit 1; }; }
need docker
need kubectl
command -v helm >/dev/null || brew install helm
kubectl cluster-info >/dev/null

echo "stopping compose listeners (Docker data plane stays in volumes)…"
docker compose -f "$ROOT/docker/docker-compose.yml" stop >/dev/null 2>&1 || true

echo "building ems + ims-processor + frontend images…"
docker build -f "$ROOT/backend/Dockerfile" -t ems:latest "$ROOT/backend"
docker build -f "$ROOT/backend/Dockerfile.ims" -t ims-processor:latest "$ROOT/backend"
docker build -f "$ROOT/frontend/Dockerfile" -t frontend:latest "$ROOT/frontend"

# Docker Desktop Kubernetes is a kind node — host images are not visible until imported.
if docker inspect desktop-control-plane >/dev/null 2>&1; then
  echo "loading images into desktop-control-plane…"
  for img in ems:latest ims-processor:latest frontend:latest; do
    docker save "$img" | docker exec -i desktop-control-plane ctr -n k8s.io images import -
  done
fi

echo "helm upgrade --install ${RELEASE} (${NS})…"
helm upgrade --install "$RELEASE" "$CHART" \
  --namespace "$NS" --create-namespace \
  -f "$CHART/values-local.yaml" \
  --server-side=false \
  --timeout 12m \
  --wait=hookOnly

echo "waiting for pods…"
kubectl -n "$NS" wait --for=condition=available --timeout=600s \
  deploy -l app.kubernetes.io/instance="$RELEASE" || true
kubectl -n "$NS" get pods -o wide

echo "port-forward (leave this running)…"
exec "$ROOT/scripts/k8s-pf.sh"
