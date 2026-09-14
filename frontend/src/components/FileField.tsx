import { useCallback, useId, useRef, useState } from "react";
import { formatBytes } from "../ui/formatBytes";

type Props = {
  file: File | null;
  disabled?: boolean;
  onChange: (file: File | null) => void;
};

export function FileField({ file, disabled, onChange }: Props) {
  const inputId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);

  const takeFile = useCallback(
    (list: FileList | null) => {
      const next = list?.[0] ?? null;
      if (next) onChange(next);
    },
    [onChange],
  );

  return (
    <div
      role="button"
      tabIndex={disabled ? -1 : 0}
      onKeyDown={(e) => {
        if (!disabled && (e.key === "Enter" || e.key === " ")) {
          e.preventDefault();
          inputRef.current?.click();
        }
      }}
      onClick={() => !disabled && inputRef.current?.click()}
      onDragEnter={(e) => {
        e.preventDefault();
        if (!disabled) setDragging(true);
      }}
      onDragOver={(e) => e.preventDefault()}
      onDragLeave={() => setDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        if (!disabled) takeFile(e.dataTransfer.files);
      }}
      className={`group relative flex min-h-[220px] cursor-pointer flex-col items-center justify-center gap-3 rounded-yt border-2 border-dashed px-6 py-10 text-center transition duration-200 ${
        disabled ? "cursor-not-allowed opacity-60" : ""
      } ${
        dragging
          ? "scale-[1.01] border-accent bg-accent-soft shadow-yt"
          : file
            ? "border-ok/40 bg-ok-soft/40"
            : "border-line-strong bg-surface hover:border-ink/30 hover:bg-paper-deep/60"
      }`}
    >
      <input
        ref={inputRef}
        id={inputId}
        type="file"
        className="sr-only"
        accept="video/mp4,video/webm,video/quicktime,video/x-matroska,.mp4,.webm,.mov,.mkv"
        disabled={disabled}
        onChange={(e) => takeFile(e.target.files)}
      />
      <span
        className={`flex h-14 w-14 items-center justify-center rounded-full bg-paper-deep text-ink-muted transition group-hover:bg-accent-soft group-hover:text-accent ${
          dragging || file ? "animate-soft-pulse bg-accent-soft text-accent" : ""
        }`}
        aria-hidden
      >
        <svg viewBox="0 0 24 24" className="h-7 w-7 fill-current">
          <path d="M9 16h6v-6h4l-7-7-7 7h4v6zm-4 2h14v2H5v-2z" />
        </svg>
      </span>
      {file ? (
        <>
          <p className="m-0 max-w-md truncate text-[16px] font-semibold text-ink">
            {file.name}
          </p>
          <p className="m-0 text-[13px] text-ink-muted">
            {formatBytes(file.size)}
            {file.type ? ` · ${file.type}` : ""}
          </p>
          <p className="m-0 text-[13px] font-medium text-ok">
            Ready to upload — click to replace
          </p>
        </>
      ) : (
        <>
          <p className="m-0 text-[18px] font-semibold text-ink">
            Drag and drop video files to upload
          </p>
          <p className="m-0 max-w-sm text-[13px] text-ink-muted">
            Your videos will go private until you publish. MP4, WebM, MOV, or MKV.
          </p>
          <span className="mt-1 rounded-yt-sm bg-paper-deep px-4 py-2 text-[13px] font-semibold text-ink">
            Select files
          </span>
        </>
      )}
    </div>
  );
}
