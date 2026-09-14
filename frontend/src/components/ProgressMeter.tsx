import { formatBytes } from "../ui/formatBytes";
import type { UploadPhase } from "../upload/runUpload";

const LABELS: Record<UploadPhase, string> = {
  idle: "Ready",
  requesting: "Preparing upload…",
  transferring: "Uploading…",
  paused: "Paused",
  completing: "Processing…",
  done: "Upload complete",
  error: "Upload failed",
  aborted: "Cancelled",
};

type Props = {
  phase: UploadPhase;
  bytesSent: number;
  totalBytes: number;
};

export function ProgressMeter({ phase, bytesSent, totalBytes }: Props) {
  if (phase === "idle") return null;
  const pct =
    totalBytes > 0 ? Math.min(100, Math.round((bytesSent / totalBytes) * 100)) : 0;
  const showBar =
    phase === "transferring" ||
    phase === "paused" ||
    phase === "completing" ||
    phase === "done" ||
    phase === "requesting";
  const width =
    phase === "completing" || phase === "done"
      ? 100
      : phase === "requesting"
        ? 8
        : pct;

  return (
    <div className="mt-6 animate-rise" aria-live="polite">
      <div className="mb-2 flex items-center justify-between gap-3 text-[13px]">
        <span className="font-semibold text-ink">{LABELS[phase]}</span>
        {phase !== "requesting" && showBar ? (
          <span className="tabular-nums text-ink-muted">
            {formatBytes(bytesSent)} / {formatBytes(totalBytes)} · {pct}%
          </span>
        ) : null}
      </div>
      {showBar ? (
        <div className="h-1.5 overflow-hidden rounded-full bg-paper-deep">
          <div
            className={`h-full rounded-full transition-[width] duration-200 ease-out ${
              phase === "paused"
                ? "bg-warn"
                : phase === "done"
                  ? "bg-ok"
                  : "bg-accent bg-[length:200%_100%] animate-shimmer"
            }`}
            style={{
              width: `${width}%`,
              backgroundImage:
                phase === "paused" || phase === "done"
                  ? undefined
                  : "linear-gradient(90deg, #ff0033 0%, #ff6680 45%, #ff0033 100%)",
            }}
          />
        </div>
      ) : null}
    </div>
  );
}
