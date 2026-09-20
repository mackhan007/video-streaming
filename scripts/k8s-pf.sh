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
kubectl -n "$NS" port-forward "svc/${NAME}-localstack" 4566:4566 &
kubectl -n "$NS" port-forward "svc/${NAME}-nginx" 8081:80 &
kubectl -n "$NS" port-forward "svc/${NAME}-pgweb" 8082:8081 &
kubectl -n "$NS" port-forward "svc/${NAME}-localstack-ui" 9008:8080 &
sleep 1
echo "leave this terminal open:"
echo "  Uploader  http://127.0.0.1:5173 (via LoadBalancer, no forward needed)"
echo "  EMS       http://127.0.0.1:8080/health"
echo "  S3 API    http://127.0.0.1:4566"
echo "  CDN       http://127.0.0.1:8081"
echo "  pgweb     http://127.0.0.1:8082"
echo "  StackPort http://127.0.0.1:9008"
wait
