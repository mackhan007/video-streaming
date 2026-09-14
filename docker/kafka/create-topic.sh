#!/usr/bin/env bash
# Ensure video.uploaded has enough partitions for parallel IMS consumers.
set -euo pipefail
BOOT="${KAFKA_BOOTSTRAP:-kafka:29092}"
TOPIC="${KAFKA_TOPIC:-video.uploaded}"
PARTS="${KAFKA_PARTITIONS:-12}"
BIN=/opt/kafka/bin/kafka-topics.sh
"$BIN" --bootstrap-server "$BOOT" --create --if-not-exists \
  --topic "$TOPIC" --partitions "$PARTS" --replication-factor 1
"$BIN" --bootstrap-server "$BOOT" --alter --topic "$TOPIC" --partitions "$PARTS" || true
"$BIN" --bootstrap-server "$BOOT" --describe --topic "$TOPIC"
