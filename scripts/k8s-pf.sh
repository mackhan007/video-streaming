#!/usr/bin/env bash
# Keep host ports mapped to the ClusterIP services. Leave this process running.
#   ./scripts/k8s-pf.sh
set -euo pipefail
export PATH="/opt/homebrew/bin:${PATH}"
NS="${K8S_NAMESPACE:-streaming}"
NAME="streaming"

cleanup() {
  trap - INT TERM EXIT
  for pid in $(jobs -p); do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
}
trap cleanup INT TERM EXIT

kubectl -n "$NS" port-forward "svc/${NAME}-ems" 8080:8080 &
kubectl -n "$NS" port-forward "svc/${NAME}-nginx" 8081:80 &
kubectl -n "$NS" port-forward "svc/${NAME}-pgweb" 8082:8081 &

# S3 backend is MinIO by default; LocalStack when localstack.enabled=true.
# Both S3 services are LoadBalancer in values-local.yaml — Docker Desktop
# already exposes them on localhost, so a port-forward would just fail to
# bind the same host port. Fall back to forwarding only if it's ClusterIP.
if kubectl -n "$NS" get svc "${NAME}-minio" >/dev/null 2>&1; then
  S3_PORT=9000; S3_LABEL="S3 API    http://127.0.0.1:9000"
  UI_LABEL="MinIO UI  http://127.0.0.1:9001 (minioadmin / minioadmin)"
  S3_TYPE="$(kubectl -n "$NS" get svc "${NAME}-minio" -o jsonpath='{.spec.type}')"
  if [ "$S3_TYPE" != "LoadBalancer" ]; then
    kubectl -n "$NS" port-forward "svc/${NAME}-minio" 9000:9000 9001:9001 &
    S3_LABEL="${S3_LABEL} (via port-forward)"
  else
    S3_LABEL="${S3_LABEL} (via LoadBalancer, no forward needed)"
  fi
else
  S3_PORT=4566
  kubectl -n "$NS" port-forward "svc/${NAME}-localstack" 4566:4566 &
  kubectl -n "$NS" port-forward "svc/${NAME}-localstack-ui" 9008:8080 &
  S3_LABEL="S3 API    http://127.0.0.1:4566"
  UI_LABEL="StackPort http://127.0.0.1:9008"
fi

sleep 1
echo "leave this terminal open:"
echo "  Uploader  http://127.0.0.1:5173 (via LoadBalancer, no forward needed)"
echo "  EMS       http://127.0.0.1:8080/health"
echo "  ${S3_LABEL}"
echo "  CDN       http://127.0.0.1:8081"
echo "  pgweb     http://127.0.0.1:8082"
echo "  ${UI_LABEL}"
wait
