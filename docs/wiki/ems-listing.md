# EMS listing

Implemented. Used by the gateway and standalone `ems-listing`.

## REST surface

| Method | Path | Action |
|---|---|---|
| `GET` | `/lister/videos?limit=&seen=` | All live uploads (any status), newest first |
| `GET` | `/lister/links?limit=&seen=` | Ready videos with `master_playlist_url` |

`limit` default **20**, max **50**. `seen` is the last `file_id` from the previous page (keyset on `(created_at DESC, id DESC)`).

```json
{
  "items": [
    {
      "file_id": "…",
      "title": "…",
      "status": "ready",
      "file_size": 123,
      "created_at": "…",
      "updated_at": "…",
      "playback_path": "hls/…/master.m3u8",
      "master_playlist_url": "http://localhost:8081/videos/hls/…/master.m3u8"
    }
  ],
  "next_seen": "…"
}
```

`master_playlist_url` is only set on `/lister/links`. `next_seen` is omitted/`null` when there is no further page.

## Key files

| Concern | Path |
|---|---|
| Routes / DTOs | `backend/ems/listing/src/api/routes.rs`, `dto.rs` |
| Use case | `app/list_page.rs` |
| Port | `ports/mod.rs` (`VideoCatalog`) |
| Adapter | `adapters/postgres.rs` |
| Indexes | `backend/ems/upload/migrations/20240914000012_listing_cursors.sql` |
| Composition | `backend/ems/listing/src/lib.rs` (`build_router`) |

## Standalone

`cargo run -p ems-listing --bin ems-listing` → `LISTING_HTTP_PORT` (8086).  
Tests: `cargo test -p ems-listing`.
