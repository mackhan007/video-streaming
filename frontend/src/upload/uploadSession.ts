import type { CompleteUploadResponse } from "../api/types";
import type { UploadPhase } from "./runUpload";

const KEY = "macky.upload.session.v1";

export type StoredUploadSession = {
  fileId: string;
  title: string;
  phase: UploadPhase;
  result: CompleteUploadResponse | null;
  fileName: string | null;
  totalBytes: number;
  bytesSent: number;
  error: string | null;
};

export function loadUploadSession(): StoredUploadSession | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as StoredUploadSession;
    if (!parsed?.fileId || typeof parsed.fileId !== "string") return null;
    return parsed;
  } catch {
    return null;
  }
}

export function saveUploadSession(session: StoredUploadSession): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(session));
  } catch {
    // quota / private mode — ignore
  }
}

export function clearUploadSession(): void {
  try {
    localStorage.removeItem(KEY);
  } catch {
    // ignore
  }
}

const MID = new Set([
  "requesting",
  "transferring",
  "paused",
  "completing",
]);

/** Map stored session → UI state (`File` cannot be restored). */
export function sessionToUploaderState(s: StoredUploadSession): {
  phase: UploadPhase;
  title: string;
  file: null;
  fileId: string;
  bytesSent: number;
  totalBytes: number;
  result: CompleteUploadResponse | null;
  error: string | null;
} {
  const interrupted = MID.has(s.phase);
  return {
    phase: interrupted ? "error" : s.phase,
    title: s.title,
    file: null,
    fileId: s.fileId,
    bytesSent: s.bytesSent,
    totalBytes: s.totalBytes,
    result: s.result,
    error: interrupted
      ? "Page reloaded during upload — Clear, then upload the same video again."
      : s.error,
  };
}
