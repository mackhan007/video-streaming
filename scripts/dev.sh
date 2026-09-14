#!/usr/bin/env bash
# Run EMS API (:8080) + React uploader (:5173) together.
# Usage (from repo root):
#   ./scripts/dev.sh
#   npm run dev
#
# Requires: rustup cargo, Node/npm, and infra (docker compose) already up.
# Loads .env via docs/local-setup/export.sh.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck disable=SC1091
source "$ROOT/docs/local-setup/export.sh"

EMS_PID=""
FE_PID=""

cleanup() {
  trap - INT TERM EXIT
  echo ""
  echo "stopping…"
  if [[ -n "${FE_PID}" ]] && kill -0 "$FE_PID" 2>/dev/null; then
    kill "$FE_PID" 2>/dev/null || true
  fi
  if [[ -n "${EMS_PID}" ]] && kill -0 "$EMS_PID" 2>/dev/null; then
    kill "$EMS_PID" 2>/dev/null || true
  fi
  wait 2>/dev/null || true
}

trap cleanup INT TERM EXIT

if [[ ! -d "$ROOT/frontend/node_modules" ]]; then
  echo "installing frontend deps…"
  (cd "$ROOT/frontend" && npm install)
fi

echo "starting EMS (cargo run -p ems-server --bin ems)…"
(
  cd "$ROOT/backend"
  cargo run -p ems-server --bin ems
) &
EMS_PID=$!

echo "starting frontend (vite :5173)…"
(
  cd "$ROOT/frontend"
  npm run dev -- --host 127.0.0.1 --port 5173
) &
FE_PID=$!

echo ""
echo "  EMS API     http://localhost:${EMS_HTTP_PORT:-8080}"
echo "  Uploader UI http://localhost:5173"
echo "  Ctrl+C to stop both"
echo ""

# Exit if either child dies.
while kill -0 "$EMS_PID" 2>/dev/null && kill -0 "$FE_PID" 2>/dev/null; do
  sleep 1
done

if ! kill -0 "$EMS_PID" 2>/dev/null; then
  echo "EMS exited unexpectedly" >&2
  exit 1
fi
if ! kill -0 "$FE_PID" 2>/dev/null; then
  echo "frontend exited unexpectedly" >&2
  exit 1
fi
