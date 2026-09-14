#!/usr/bin/env bash
# Run EMS API (:8080) + IMS processor (:8088) + React uploader (:5173).
# Usage (from repo root):
#   ./scripts/dev.sh
#   npm run dev
#
# Requires: rustup cargo, Node/npm, ffmpeg, and infra (docker compose) already up.
# Loads .env via docs/local-setup/export.sh.
# IMS_WORKERS (default 3) extra processors on PROCESSOR_HTTP_PORT, +1, +2, …

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Cursor/sandbox may inject CARGO_TARGET_DIR — keep builds in backend/target.
unset CARGO_TARGET_DIR || true
export PATH="${HOME}/.cargo/bin:${PATH}"

# shellcheck disable=SC1091
source "$ROOT/docs/local-setup/export.sh"

EMS_PID=""
IMS_PIDS=()
FE_PID=""

cleanup() {
  trap - INT TERM EXIT
  echo ""
  echo "stopping…"
  for pid in "$FE_PID" "${IMS_PIDS[@]:-}" "$EMS_PID"; do
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

echo "building EMS + IMS…"
(cd "$ROOT/backend" && cargo build -p ems-server --bin ems -p ims-processor --bin ims-processor)
EMS_BIN="$ROOT/backend/target/debug/ems"
IMS_BIN="$ROOT/backend/target/debug/ims-processor"

echo "starting EMS…"
"$EMS_BIN" &
EMS_PID=$!

WORKERS="${IMS_WORKERS:-3}"
echo "starting IMS processor × ${WORKERS}…"
base_port="${PROCESSOR_HTTP_PORT:-8088}"
work_root="${IMS_WORK_DIR:-/tmp/ims-processor}"
for i in $(seq 0 $((WORKERS - 1))); do
  port=$((base_port + i))
  PROCESSOR_HTTP_PORT="$port" \
    LOG_NAME="ims-processor-${port}" \
    IMS_WORK_DIR="${work_root}-${port}" \
    "$IMS_BIN" &
  IMS_PIDS+=($!)
done

echo "starting frontend (vite :5173)…"
(cd "$ROOT/frontend" && npm run dev -- --host 127.0.0.1 --port 5173) &
FE_PID=$!

echo ""
echo "  EMS API     http://localhost:${EMS_HTTP_PORT:-8080}"
echo "  IMS health  http://localhost:${base_port}/health  (${WORKERS} workers)"
echo "  Uploader UI http://localhost:5173"
echo "  Ctrl+C to stop all"
echo ""

ims_up() {
  for pid in "${IMS_PIDS[@]}"; do
    kill -0 "$pid" 2>/dev/null || return 1
  done
  return 0
}

while kill -0 "$EMS_PID" 2>/dev/null \
  && ims_up \
  && kill -0 "$FE_PID" 2>/dev/null; do
  sleep 1
done

echo "a process exited unexpectedly" >&2
exit 1
