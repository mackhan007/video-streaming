import { ActionBar } from "./ActionBar";
import { FileField } from "./FileField";
import { ProgressMeter } from "./ProgressMeter";
import { StatusBanner } from "./StatusBanner";
import { TitleField } from "./TitleField";
import { useVideoUpload } from "../hooks/useVideoUpload";

const LOCKED = new Set(["requesting", "transferring", "paused", "completing"]);

export function UploadWorkbench() {
  const { state, setTitle, setFile, start, pause, resume, abort, reset } =
    useVideoUpload();
  const locked = LOCKED.has(state.phase);

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
          Drop a file to start. Pause anytime — resume continues where you left
          off.
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
          />
        </div>
      </div>
    </section>
  );
}
