import { ActionBar } from "./ActionBar";
import { FileField } from "./FileField";
import { PipelineSteps } from "./PipelineSteps";
import { ProgressMeter } from "./ProgressMeter";
import { StatusBanner } from "./StatusBanner";
import { TitleField } from "./TitleField";
import { VideoPlayer } from "./VideoPlayer";
import { usePipeline } from "../hooks/usePipeline";
import { useVideoUpload } from "../hooks/useVideoUpload";
import { btnPrimary } from "../ui/classes";

const LOCKED = new Set(["requesting", "transferring", "paused", "completing"]);

export function UploadWorkbench() {
  const { state, setTitle, setFile, start, pause, resume, abort, reset } =
    useVideoUpload();
  const locked = LOCKED.has(state.phase);
  const pipeline = usePipeline(state.fileId);

  return (
    <section
      className="animate-rise-delay mx-auto w-full max-w-3xl px-4 pb-16 pt-8 sm:px-6"
      aria-labelledby="upload-heading"
    >
      <div className="mb-8">
        <h1
          id="upload-heading"
          className="m-0 text-[28px] font-extrabold tracking-tight text-ink sm:text-[32px]"
        >
          Upload videos
        </h1>
        <p className="mt-2 max-w-xl text-[15px] text-ink-muted">
          Watch each stage live: upload, Kafka queue, IMS processing, ready, then
          stream from the CDN.
        </p>
      </div>

      <div className="rounded-yt bg-surface p-5 shadow-yt sm:p-7">
        <FileField file={state.file} disabled={locked} onChange={setFile} />
        <div className="mt-6 border-t border-line pt-6">
          <TitleField
            value={state.title}
            disabled={locked}
            onChange={setTitle}
          />
          <ActionBar
            phase={state.phase}
            canUpload={Boolean(state.file)}
            onStart={() => void start()}
            onPause={pause}
            onResume={resume}
            onAbort={abort}
            onReset={reset}
          />
          <ProgressMeter
            phase={state.phase}
            bytesSent={state.bytesSent}
            totalBytes={state.totalBytes}
          />
          <StatusBanner
            phase={state.phase}
            fileId={state.fileId}
            result={state.result}
            error={state.error}
            liveStatus={pipeline.status?.status}
          />
          <PipelineSteps steps={pipeline.steps} uploadPhase={state.phase} />
          {pipeline.error ? (
            <div className="mt-4 flex flex-wrap items-center gap-3">
              <p className="m-0 text-[13px] text-danger">{pipeline.error}</p>
              {pipeline.failed ? (
                <button
                  type="button"
                  className={btnPrimary}
                  disabled={pipeline.retrying}
                  onClick={() => void pipeline.retry()}
                >
                  {pipeline.retrying ? "Retrying…" : "Retry processing"}
                </button>
              ) : null}
            </div>
          ) : null}
          {pipeline.status ? (
            <p className="mt-3 text-[13px] text-ink-muted">
              DB status:{" "}
              <strong className="text-ink">{pipeline.status.status}</strong>
              {pipeline.status.playback_path
                ? ` · ${pipeline.status.playback_path}`
                : ""}
            </p>
          ) : null}
          {pipeline.stream ? (
            <VideoPlayer
              src={pipeline.stream.master_playlist_url}
              title={pipeline.status?.title}
              fileId={pipeline.stream.file_id}
            />
          ) : null}
        </div>
      </div>
    </section>
  );
}
