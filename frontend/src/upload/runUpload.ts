import { uploadApi } from "../api/uploadApi";
import type { CompleteUploadResponse } from "../api/types";
import {
  putToStorage,
  TransferAbortedError,
  TransferControl,
  type PutProgress,
} from "./putToStorage";

export type UploadPhase =
  | "idle"
  | "requesting"
  | "transferring"
  | "paused"
  | "completing"
  | "done"
  | "error"
  | "aborted";

export type RunUploadCallbacks = {
  onPhase: (phase: UploadPhase) => void;
  onProgress: (p: PutProgress) => void;
  onFileId: (fileId: string) => void;
};

const ALLOWED = new Set([
  "video/mp4",
  "video/webm",
  "video/quicktime",
  "video/x-matroska",
]);

export function pickContentType(file: File): string | undefined {
  if (file.type && ALLOWED.has(file.type)) return file.type;
  const name = file.name.toLowerCase();
  if (name.endsWith(".mp4")) return "video/mp4";
  if (name.endsWith(".webm")) return "video/webm";
  if (name.endsWith(".mov")) return "video/quicktime";
  if (name.endsWith(".mkv")) return "video/x-matroska";
  return file.type || undefined;
}

/** Application use case: create → PUT storage → complete. */
export async function runUpload(
  file: File,
  title: string,
  control: TransferControl,
  cb: RunUploadCallbacks,
): Promise<CompleteUploadResponse> {
  cb.onPhase("requesting");
  const session = await uploadApi.create({
    fileSize: file.size,
    title: title.trim() || undefined,
    contentType: pickContentType(file),
  });
  cb.onFileId(session.file_id);

  if (control.aborted) {
    await uploadApi.abort(session.file_id).catch(() => undefined);
    cb.onPhase("aborted");
    throw new TransferAbortedError();
  }

  cb.onPhase("transferring");
  try {
    await putToStorage(file, session, cb.onProgress, control);
  } catch (e) {
    if (e instanceof TransferAbortedError || control.aborted) {
      await uploadApi.abort(session.file_id).catch(() => undefined);
      cb.onPhase("aborted");
      throw e instanceof TransferAbortedError ? e : new TransferAbortedError();
    }
    await uploadApi.abort(session.file_id).catch(() => undefined);
    throw e;
  }

  if (control.aborted) {
    await uploadApi.abort(session.file_id).catch(() => undefined);
    cb.onPhase("aborted");
    throw new TransferAbortedError();
  }

  cb.onPhase("completing");
  const done = await uploadApi.complete(session.file_id);
  cb.onPhase("done");
  return done;
}

export { TransferControl };
