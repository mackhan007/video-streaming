import type { CompleteUploadResponse } from "../api/types";
import {
  bannerErr,
  bannerNeutral,
  bannerOk,
  bannerWarn,
} from "../ui/classes";
import type { UploadPhase } from "../upload/runUpload";

type Props = {
  phase: UploadPhase;
  fileId: string | null;
  result: CompleteUploadResponse | null;
  error: string | null;
  liveStatus?: string | null;
};

export function StatusBanner({ phase, fileId, result, error, liveStatus }: Props) {
  if (phase === "done" && result) {
    const status = liveStatus ?? result.status;
    const encoding = status === "processing";
    return (
      <div className={`${bannerOk} animate-rise`} role="status">
        <span aria-hidden>✓</span>
        <span>
          {encoding
            ? "Upload complete. IMS is encoding ABR HLS. Status "
            : "Your video finished uploading. Status "}
          <strong className="font-semibold">{status}</strong>
          <span className="mt-1 block font-mono text-[12px] opacity-80">
            {result.file_id}
          </span>
        </span>
      </div>
    );
  }
  if (phase === "error" && error) {
    return (
      <div className={`${bannerErr} animate-rise`} role="alert">
        <span aria-hidden>!</span>
        <span>
          {error}
          {fileId ? (
            <span className="mt-1 block font-mono text-[12px] opacity-80">
              {fileId}
            </span>
          ) : null}
        </span>
      </div>
    );
  }
  if (phase === "aborted") {
    return (
      <div className={`${bannerNeutral} animate-rise`} role="status">
        Upload cancelled
        {fileId ? `. Session ${fileId}` : "."}
      </div>
    );
  }
  if (phase === "paused" && fileId) {
    return (
      <div className={`${bannerWarn} animate-rise`} role="status">
        Upload paused. Hit Resume to continue from the unfinished part.
      </div>
    );
  }
  return null;
}
