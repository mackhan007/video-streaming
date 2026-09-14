# syntax=docker/dockerfile:1
# Build the unified EMS server (and workspace) with the official Rust image.
# Usage from repo root:
#   docker build -f backend/Dockerfile -t ems:latest backend

FROM rust:latest AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    g++ \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN --mount=type=cache,id=cargo-reg,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=ems-target,target=/app/target \
    cargo build --release -p ems-server \
    && cp /app/target/release/ems /ems

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3t64 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /ems /usr/local/bin/ems

ENV EMS_HTTP_PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/ems"]
