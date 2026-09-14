import { useEffect, useState } from "react";
import type { PipelineStep, PipelineStepState } from "../api/types";
import { formatDuration, stepDurationMs } from "../ui/formatDuration";

type Props = {
  steps: PipelineStep[];
  uploadPhase: string;
};

const CORE: { id: string; label: string; hint: string }[] = [
  { id: "upload", label: "Upload", hint: "Browser → S3" },
  { id: "queue", label: "Queue", hint: "Kafka event" },
  { id: "process", label: "Process", hint: "IMS · FFmpeg HLS" },
  { id: "ready", label: "Ready", hint: "Postgres" },
  { id: "play", label: "Stream", hint: "CDN playlist" },
];

const ABR: { id: string; label: string; hint: string }[] = [
  { id: "hls_360", label: "360p", hint: "ABR rung" },
  { id: "hls_720", label: "720p", hint: "ABR rung" },
  { id: "hls_1080", label: "1080p", hint: "ABR rung" },
];

function useNow(active: boolean) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return;
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, [active]);
  return now;
}

function uiState(
  step: PipelineStep | undefined,
  id: string,
  uploadPhase: string,
): "todo" | "active" | "done" | "error" {
  if (step) {
    const map: Record<PipelineStepState, "todo" | "active" | "done" | "error"> =
      {
        pending: "todo",
        running: "active",
        done: "done",
        failed: "error",
      };
    return map[step.state];
  }
  if (id === "upload") {
    if (uploadPhase === "done") return "done";
    if (
      ["requesting", "transferring", "paused", "completing"].includes(
        uploadPhase,
      )
    ) {
      return "active";
    }
  }
  return "todo";
}

function StepCard(props: {
  index: number;
  meta: { id: string; label: string; hint: string };
  steps: PipelineStep[];
  uploadPhase: string;
  now: number;
}) {
  const row = props.steps.find((s) => s.step === props.meta.id);
  const st = uiState(row, props.meta.id, props.uploadPhase);
  const caption = row?.error || row?.detail || st;
  const ms = stepDurationMs(
    row?.started_at ?? null,
    row?.finished_at ?? null,
    st === "active",
    props.now,
  );
  const elapsed = ms == null ? null : formatDuration(ms);
  return (
    <li
      className={`relative rounded-yt-sm border px-3 py-3 transition ${
        st === "done"
          ? "border-ok/40 bg-ok-soft/50"
          : st === "active"
            ? "border-accent/50 bg-accent-soft shadow-yt"
            : st === "error"
              ? "border-danger/40 bg-danger-soft"
              : "border-line bg-paper-deep/40"
      }`}
    >
      <p className="m-0 text-[11px] font-semibold uppercase tracking-wide text-ink-soft">
        {props.index}. {props.meta.label}
      </p>
      <p className="m-0 mt-1 text-[13px] font-semibold text-ink">
        {props.meta.hint}
      </p>
      <p className="m-0 mt-1 truncate text-[12px] text-ink-muted">{caption}</p>
      {elapsed ? (
        <p className="m-0 mt-1 tabular-nums text-[12px] font-semibold text-ink">
          {st === "active" ? elapsed : `took ${elapsed}`}
        </p>
      ) : null}
    </li>
  );
}

export function PipelineSteps({ steps, uploadPhase }: Props) {
  const ticking = steps.some((s) => s.state === "running") ||
    ["requesting", "transferring", "paused", "completing"].includes(uploadPhase);
  const now = useNow(ticking);
  return (
    <div className="mt-6 space-y-3">
      <ol className="grid gap-2 sm:grid-cols-5">
        {CORE.map((m, i) => (
          <StepCard
            key={m.id}
            index={i + 1}
            meta={m}
            steps={steps}
            uploadPhase={uploadPhase}
            now={now}
          />
        ))}
      </ol>
      <p className="m-0 text-[11px] font-semibold uppercase tracking-wide text-ink-soft">
        Adaptive bitrate
      </p>
      <ol className="grid gap-2 sm:grid-cols-3">
        {ABR.map((m, i) => (
          <StepCard
            key={m.id}
            index={i + 1}
            meta={m}
            steps={steps}
            uploadPhase={uploadPhase}
            now={now}
          />
        ))}
      </ol>
    </div>
  );
}
