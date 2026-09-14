# Frontend — upload desk

React (Vite) UI for EMS upload: create → PUT S3 → complete.

## Run

```bash
# Infra first
docker compose -f docker/docker-compose.yml up -d

# EMS + IMS × IMS_WORKERS + uploader
npm run dev
# or: ./scripts/dev.sh
```

Open http://localhost:5173 — Vite proxies `/uploader` to `http://localhost:8080`.

`IMS_WORKERS` (default 3) extra IMS binaries on `:8088`, `:8089`, … The health chip still probes `:8088`.

Manual (two terminals): EMS `cargo run -p ems-server --bin ems` in `backend/`, then `npm run dev` in `frontend/`.

Env: `.env.example` (`VITE_API_BASE_URL` empty in dev).

## Layout

| Path | Role |
|---|---|
| `src/api/` | HTTP client + upload REST |
| `src/upload/` | PUT storage + pause/resume control + `runUpload` |
| `src/hooks/useVideoUpload.ts` | UI upload state (+ `localStorage` session) |
| `src/upload/uploadSession.ts` | Persist / restore `fileId` session across reload |
| `src/hooks/usePipeline.ts` | Polls `GET …/pipeline` (+ status/stream) |
| `src/components/` | YouTube-studio style upload UI (Tailwind) |
| `src/components/PipelineSteps.tsx` | Stepper driven by `video_pipeline_steps` |
| `src/index.css` | Tailwind entry + base layer |
| `src/ui/` | Shared class strings + helpers (`formatBytes`, `formatDuration`) |
| `tailwind.config.js` | Classic desk theme tokens |

Styling: **Tailwind CSS v3** (PostCSS). Theme colors/fonts live in `tailwind.config.js`.

UI shows live **EMS / IMS / CDN** health chips and a **pipeline stepper** fed from `GET /uploader/videos/{id}/pipeline` (`video_pipeline_steps`), including an **ABR row** (360p / 720p / 1080p) with **% complete** (`progress_pct`, `chunks_done` / `chunks_total`) and **elapsed / took** per step (`started_at`–`finished_at`), and HLS playback when the ready step is done. After upload, the banner uses live DB status (`processing` while IMS encodes), not the frozen complete-upload payload.

Allowed types: mp4 / webm / quicktime / matroska (same as EMS). Header sniff (`src/upload/assertVideoFile.ts`) accepts ISO BMFF `ftyp` **and** `mdat`-first files. A `.mp4` that is actually a ZIP of stored videos (`src/upload/unwrapZipVideo.ts`) is unwrapped (name-matched or single entry); multi-video archives ask the user to unzip.

**Pause / resume:** During transfer, **Pause** aborts the in-flight S3 PUT and keeps the EMS session. **Resume** retries the unfinished part (multipart continues from that part; single-object mode restarts the one PUT). **Cancel** still aborts the upload session on EMS.

**Reload:** After a `fileId` exists, session is saved to `localStorage` so refresh keeps the pipeline / player. Mid-transfer reloads cannot restore the `File` blob — Clear and upload again. **Clear** wipes the stored session.
