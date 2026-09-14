#!/usr/bin/env bash
# Run EMS API (:8080) + IMS processor (:8088) + React uploader (:5173).
# Usage (from repo root):
#   ./scripts/dev.sh
#   npm run dev
#
# Requires: rustup cargo, Node/npm, ffmpeg, and infra (docker compose) already up.
# Loads .env via docs/local-setup/export.sh.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck disable=SC1091
source "$ROOT/docs/local-setup/export.sh"

EMS_PID=""
IMS_PID=""
FE_PID=""

cleanup() {
  trap - INT TERM EXIT
  echo ""
  echo "stopping…"
  for pid in "$FE_PID" "$IMS_PID" "$EMS_PID"; do
    if [[ -n "${pid}" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
    fi
  done
  wait 2>/dev/null || true
}

trap cleanup INT TERM EXIT

if ! command -v "${FFMPEG_PATH:-ffmpeg}" >/dev/null 2>&1; then
  echo "warning: ffmpeg not found (${FFMPEG_PATH:-ffmpeg}) — IMS HLS will fail until installed" >&2
fi

if [[ ! -d "$ROOT/frontend/node_modules" ]]; then
  echo "installing frontend deps…"
  (cd "$ROOT/frontend" && npm install)
fi

echo "starting EMS (cargo run -p ems-server --bin ems)…"
(cd "$ROOT/backend" && cargo run -p ems-server --bin ems) &
EMS_PID=$!

echo "starting IMS processor…"
(cd "$ROOT/backend" && cargo run -p ims-processor --bin ims-processor) &
IMS_PID=$!

echo "starting frontend (vite :5173)…"
(cd "$ROOT/frontend" && npm run dev -- --host 127.0.0.1 --port 5173) &
FE_PID=$!

echo ""
echo "  EMS API     http://localhost:${EMS_HTTP_PORT:-8080}"
echo "  IMS health  http://localhost:${PROCESSOR_HTTP_PORT:-8088}/health"
echo "  Uploader UI http://localhost:5173"
echo "  Ctrl+C to stop all"
echo ""

while kill -0 "$EMS_PID" 2>/dev/null \
  && kill -0 "$IMS_PID" 2>/dev/null \
  && kill -0 "$FE_PID" 2>/dev/null; do
  sleep 1
done

echo "a process exited unexpectedly" >&2
exit 1
