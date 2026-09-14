import { useCallback, useEffect, useRef, useState } from "react";
import type { CompleteUploadResponse } from "../api/types";
import {
  runUpload,
  TransferControl,
  type UploadPhase,
} from "../upload/runUpload";
import { TransferAbortedError } from "../upload/transferControl";
import {
  clearUploadSession,
  loadUploadSession,
  saveUploadSession,
  sessionToUploaderState,
} from "../upload/uploadSession";

export type UploaderState = {
  phase: UploadPhase;
  title: string;
  file: File | null;
  fileId: string | null;
  bytesSent: number;
  totalBytes: number;
  result: CompleteUploadResponse | null;
  error: string | null;
};

const initial: UploaderState = {
  phase: "idle",
  title: "",
  file: null,
  fileId: null,
  bytesSent: 0,
  totalBytes: 0,
  result: null,
  error: null,
};

function readInitial(): UploaderState {
  const stored = loadUploadSession();
  return stored ? sessionToUploaderState(stored) : initial;
}

export function useVideoUpload() {
  const boot = useRef(readInitial());
  const [state, setState] = useState<UploaderState>(boot.current);
  const draft = useRef({
    title: boot.current.title,
    file: null as File | null,
  });
  const controlRef = useRef<TransferControl | null>(null);

  useEffect(() => {
    if (!state.fileId) return;
    saveUploadSession({
      fileId: state.fileId,
      title: state.title,
      phase: state.phase,
      result: state.result,
      fileName: state.file?.name ?? null,
      totalBytes: state.totalBytes,
      bytesSent: state.bytesSent,
      error: state.error,
    });
  }, [state]);

  const setTitle = useCallback((title: string) => {
    draft.current.title = title;
    setState((s) => ({ ...s, title }));
  }, []);

  const setFile = useCallback((file: File | null) => {
    draft.current.file = file;
    setState((s) => ({
      ...s,
      file,
      totalBytes: file?.size ?? 0,
      bytesSent: 0,
      error: null,
      result: null,
      fileId: null,
      phase: "idle",
    }));
    clearUploadSession();
  }, []);

  const reset = useCallback(() => {
    controlRef.current?.abort();
    controlRef.current = null;
    draft.current = { title: "", file: null };
    clearUploadSession();
    setState(initial);
  }, []);

  const abort = useCallback(() => {
    controlRef.current?.abort();
  }, []);

  const pause = useCallback(() => {
    const c = controlRef.current;
    if (!c || c.aborted) return;
    c.pause();
    setState((s) =>
      s.phase === "transferring" ? { ...s, phase: "paused" } : s,
    );
  }, []);

  const resume = useCallback(() => {
    const c = controlRef.current;
    if (!c || c.aborted) return;
    c.resume();
    setState((s) =>
      s.phase === "paused" ? { ...s, phase: "transferring" } : s,
    );
  }, []);

  const start = useCallback(async () => {
    const file = draft.current.file;
    const title = draft.current.title;
    if (!file) {
      setState((s) => ({
        ...s,
        error: "Choose a video file first.",
        phase: "error",
      }));
      return;
    }
    const control = new TransferControl();
    controlRef.current = control;
    setState((s) => ({
      ...s,
      error: null,
      result: null,
      bytesSent: 0,
      totalBytes: file.size,
    }));

    try {
      const result = await runUpload(file, title, control, {
        onPhase: (phase) => setState((s) => ({ ...s, phase })),
        onProgress: (p) =>
          setState((s) => ({
            ...s,
            bytesSent: p.bytesSent,
            totalBytes: p.totalBytes,
          })),
        onFileId: (fileId) => setState((s) => ({ ...s, fileId })),
        onResolvedFile: (resolved) => {
          draft.current.file = resolved;
          setState((s) => ({
            ...s,
            file: resolved,
            totalBytes: resolved.size,
          }));
        },
      });
      setState((s) => ({ ...s, result, phase: "done" }));
    } catch (e) {
      const aborted = e instanceof TransferAbortedError || control.aborted;
      const msg = e instanceof Error ? e.message : "Upload failed";
      setState((s) => ({
        ...s,
        error: aborted ? null : msg,
        phase: aborted ? "aborted" : "error",
      }));
    } finally {
      controlRef.current = null;
    }
  }, []);

  return { state, setTitle, setFile, start, pause, resume, abort, reset };
}
