# Build the unified EMS server (and workspace) with the official Rust image.
# Usage from repo root:
#   docker build -f backend/Dockerfile -t ems backend
#   docker run --rm -p 8080:8080 --env-file .env \
#     -e DATABASE_URL=postgres://streaming:streaming@host.docker.internal:5432/streaming \
#     -e REDIS_URL=redis://host.docker.internal:6379 \
#     -e AWS_ENDPOINT_URL=http://host.docker.internal:4566 \
#     -e KAFKA_BOOTSTRAP_SERVERS=host.docker.internal:9092 \
#     ems

FROM rust:latest AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    g++ \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release -p ems-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ems /usr/local/bin/ems

ENV EMS_HTTP_PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/ems"]
