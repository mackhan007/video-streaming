import { btnDanger, btnGhost, btnPrimary } from "../ui/classes";
import type { UploadPhase } from "../upload/runUpload";

type Props = {
  phase: UploadPhase;
  canUpload: boolean;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onAbort: () => void;
  onReset: () => void;
};

export function ActionBar({
  phase,
  canUpload,
  onStart,
  onPause,
  onResume,
  onAbort,
  onReset,
}: Props) {
  const transferring = phase === "transferring";
  const paused = phase === "paused";
  const busy =
    phase === "requesting" || transferring || paused || phase === "completing";

  return (
    <div className="mt-6 flex flex-wrap items-center gap-2.5 max-[560px]:flex-col [&_button]:max-[560px]:w-full">
      <button
        type="button"
        className={btnPrimary}
        disabled={busy || !canUpload}
        onClick={onStart}
      >
        {busy ? "Uploading…" : "Upload"}
      </button>
      {transferring ? (
        <button type="button" className={btnGhost} onClick={onPause}>
          Pause
        </button>
      ) : null}
      {paused ? (
        <button type="button" className={btnPrimary} onClick={onResume}>
          Resume
        </button>
      ) : null}
      {busy ? (
        <button type="button" className={btnDanger} onClick={onAbort}>
          Cancel
        </button>
      ) : null}
      <button
        type="button"
        className={`${btnGhost} ml-auto max-[560px]:ml-0`}
        disabled={busy}
        onClick={onReset}
      >
        Clear
      </button>
    </div>
  );
}
