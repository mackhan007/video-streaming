# Frontend — upload desk

React (Vite) UI for EMS upload: create → PUT S3 → complete.

## Run

```bash
# Infra first
docker compose -f docker/docker-compose.yml up -d

# EMS + uploader together
npm run dev
# or: ./scripts/dev.sh
```

Open http://localhost:5173 — Vite proxies `/uploader` to `http://localhost:8080`.

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
| `src/ui/` | Shared class strings + helpers |
| `tailwind.config.js` | Classic desk theme tokens |

Styling: **Tailwind CSS v3** (PostCSS). Theme colors/fonts live in `tailwind.config.js`.

UI shows live **EMS / IMS / CDN** health chips and a **pipeline stepper** fed from `GET /uploader/videos/{id}/pipeline` (`video_pipeline_steps`), including an **ABR row** (360p / 720p / 1080p), with HLS playback when the ready step is done.

Allowed types: mp4 / webm / quicktime / matroska (same as EMS).

**Pause / resume:** During transfer, **Pause** aborts the in-flight S3 PUT and keeps the EMS session. **Resume** retries the unfinished part (multipart continues from that part; single-object mode restarts the one PUT). **Cancel** still aborts the upload session on EMS.

**Reload:** After a `fileId` exists, session is saved to `localStorage` so refresh keeps the pipeline / player. Mid-transfer reloads cannot restore the `File` blob — Clear and upload again. **Clear** wipes the stored session.
