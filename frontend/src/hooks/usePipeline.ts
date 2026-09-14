import { useCallback, useEffect, useRef, useState } from "react";
import { uploadApi } from "../api/uploadApi";
import type {
  PipelineStep,
  StreamResponse,
  VideoStatusResponse,
} from "../api/types";

function stepOf(steps: PipelineStep[], name: string) {
  return steps.find((s) => s.step === name);
}

/** Poll `video_pipeline_steps` via EMS; fetch stream when ready. */
export function usePipeline(fileId: string | null) {
  const [steps, setSteps] = useState<PipelineStep[]>([]);
  const [status, setStatus] = useState<VideoStatusResponse | null>(null);
  const [stream, setStream] = useState<StreamResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [retrying, setRetrying] = useState(false);
  const [pollKey, setPollKey] = useState(0);
  const timer = useRef<number | null>(null);
  const streamFetched = useRef(false);

  useEffect(() => {
    setSteps([]);
    setStatus(null);
    setStream(null);
    setError(null);
    streamFetched.current = false;
    if (!fileId) return;

    let alive = true;
    const tick = async () => {
      try {
        const [pipe, st] = await Promise.all([
          uploadApi.pipeline(fileId),
          uploadApi.status(fileId).catch(() => null),
        ]);
        if (!alive) return;
        setSteps(pipe.steps);
        if (st) setStatus(st);

        const process = stepOf(pipe.steps, "process");
        const ready = stepOf(pipe.steps, "ready");
        if (process?.state === "failed" || st?.status === "failed") {
          setError(process?.error || "Processing failed");
          return;
        }

        if (ready?.state === "done" && !streamFetched.current) {
          streamFetched.current = true;
          const s = await uploadApi.stream(fileId);
          if (!alive) return;
          setStream(s);
          const again = await uploadApi.pipeline(fileId);
          if (!alive) return;
          setSteps(again.steps);
          return;
        }

        const play = stepOf(pipe.steps, "play");
        if (play?.state === "done") return;

        timer.current = window.setTimeout(() => void tick(), 1500);
      } catch (e) {
        if (!alive) return;
        setError(e instanceof Error ? e.message : "Pipeline poll failed");
        timer.current = window.setTimeout(() => void tick(), 3000);
      }
    };
    void tick();
    return () => {
      alive = false;
      if (timer.current) window.clearTimeout(timer.current);
    };
  }, [fileId, pollKey]);

  const retry = useCallback(async () => {
    if (!fileId) return;
    setRetrying(true);
    setError(null);
    try {
      await uploadApi.retry(fileId);
      streamFetched.current = false;
      setStream(null);
      setPollKey((k) => k + 1);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Retry failed");
    } finally {
      setRetrying(false);
    }
  }, [fileId]);

  const failed =
    status?.status === "failed" ||
    steps.some((s) => s.step === "process" && s.state === "failed");

  return { steps, status, stream, error, failed, retrying, retry };
}
