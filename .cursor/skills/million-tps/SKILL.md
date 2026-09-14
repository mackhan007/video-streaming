---
name: million-tps
description: >-
  Designs and implements code for extreme throughput (target: 1 million TPS),
  including required indexing on every hot-path query. Use whenever creating,
  editing, refactoring, or reviewing backend services, data paths, APIs,
  workers, schemas, or infra on the hot path.
---

# Million TPS readiness

## Rule (verbatim)

The code should be written in such a way it should be able to take 1 million tps

indexing should also be added for same

## Mindset

Treat every hot-path change as latency- and throughput-sensitive. 1M TPS is achieved by **horizontal scale + tiny per-request work**, not by a single fat process. Code must stay safe and efficient when many replicas run in parallel. **Every query and cache key on the hot path must be backed by an index** (DB, Redis key design, or Kafka partition key).

## Hot-path rules

1. **No blocking on the request path** — async IO only; never sync disk/network in handlers
2. **Bounded work per request** — O(1) or small O(log n); no unbounded scans, large payloads, or N+1 IO
3. **Cheap allocations** — avoid cloning large strings/buffers; reuse buffers where it matters
4. **Timeouts + backpressure** — every IO call has a timeout; fail fast under overload; don’t unbounded-queue
5. **Idempotent writes** — retries and duplicate clients must not corrupt state (upload-complete style)
6. **Stateless app tier** — session/cache in Redis (or equivalent); any replica can serve any request
7. **Connection pooling** — DB/Redis/Kafka/S3 clients pooled and shared (`Arc`), not per-request connect
8. **Batch / pipeline where fan-out hurts** — prefer one round-trip over chatty loops when scaling out
9. **Avoid global locks** — no process-wide `Mutex` on the hot path; shard or use lock-free/async primitives
10. **Observability without drowning** — structured logs; keep hot-path logs at `debug`/`trace`; metrics for rate/latency/errors
11. **Index every hot lookup** — see Indexing below; no sequential scans at scale

## Indexing (required for 1M TPS)

Ship indexes in the **same change** as the query/filter that needs them. Never add a `WHERE` / `ORDER BY` / join without a matching index plan.

| Access pattern | Index approach |
|---|---|
| Point lookup by id | Primary key / unique (`videos.id`) |
| List ready videos (`status` + cursor time) | Composite `(status, created_at DESC)`; prefer **partial** index `WHERE status = 'ready'` when listing only ready |
| Lookup by object key / playback path | Unique or btree on `object_key` / `playback_path` |
| Worker claim / status transitions | Index supporting `status` (+ `updated_at` if polled) |
| Redis sessions | Key per id (`upload:{file_id}`) + **TTL** (index = keyspace design) |
| Kafka | Partition key = `file_id` (ordering + parallel consumers) |

Rules:

- Migrations: `CREATE INDEX` / `CREATE UNIQUE INDEX` (use `CONCURRENTLY` in prod when table is live)
- Composite order matches query predicates left-to-right
- Partial indexes for hot subsets (`ready`, `pending`) to keep indexes small
- Covering indexes only when proven; don’t over-index write-heavy columns
- New listing/streaming/IMS queries → new indexes in the same PR
- Verify with `EXPLAIN (ANALYZE)` mentally: index scan / index only, not seq scan

## Architecture expectations

- Split read/write paths when needed (CQRS-lite)
- Cache aggressively for read-heavy endpoints (listing); invalidate or TTL correctly
- Push heavy work off the HTTP path (Kafka → IMS processor)
- Presigned direct-to-object-store uploads (client → S3), not proxying multi‑GB through EMS
- Partition Kafka keys for parallelism; consumers must scale out
- DB: **indexed** lookups only; no table scans; connection pool sized for concurrency
- Prefer CDN for playback; origin never serves 1M media streams itself

## What not to do

- Load entire objects into memory on the API
- Synchronous “notify everyone” fan-out inside a request
- Unbounded `Vec` growth, unbounded Redis keys without TTL
- Holding DB transactions open across external HTTP/S3 calls
- Per-request thread spawn storms
- Filters/`ORDER BY` without a supporting index

## Checklist before done

- [ ] Per-request IO count is minimal and bounded
- [ ] All remote calls have timeouts; pools are shared
- [ ] Hot path is async and lock-free (no global mutex)
- [ ] Heavy work is async/event-driven off the critical path
- [ ] Design remains horizontally scalable (no sticky single-node assumption)
- [ ] Every new query/filter has a matching DB (or key) index in the same change
- [ ] Still SOLID, patterned, and ≤200 lines per file
