import type { CreateUploadResponse, PresignedPart } from "../api/types";
import { putBlob } from "./putBlob";
import {
  TransferAbortedError,
  TransferControl,
  TransferPausedError,
} from "./transferControl";

export type PutProgress = {
  bytesSent: number;
  totalBytes: number;
};

/** Match EMS: parts 1..n-1 are equal width; last is the remainder. */
export function sliceFile(file: File, parts: PresignedPart[]): Blob[] {
  const n = parts.length;
  const partSize = Math.ceil(file.size / n);
  return parts.map((_, i) => {
    const start = i * partSize;
    return file.slice(start, Math.min(file.size, start + partSize));
  });
}

async function putOneWithRetry(
  url: string,
  blob: Blob,
  contentType: string | undefined,
  control: TransferControl,
  onPartProgress: (loaded: number) => void,
): Promise<void> {
  for (;;) {
    await control.waitWhilePaused();
    control.throwIfStopped();
    try {
      await putBlob(url, blob, contentType, control, onPartProgress);
      return;
    } catch (e) {
      if (e instanceof TransferPausedError) continue;
      throw e;
    }
  }
}

/**
 * Upload file bytes. Pause aborts the in-flight PUT; resume retries that part
 * (single-object mode restarts the whole PUT).
 */
export async function putToStorage(
  file: File,
  session: CreateUploadResponse,
  onProgress: (p: PutProgress) => void,
  control: TransferControl,
): Promise<void> {
  const sorted = [...session.parts].sort((a, b) => a.part_number - b.part_number);
  if (sorted.length === 0) throw new Error("No presigned parts returned");

  if (session.mode === "single") {
    await putOneWithRetry(
      sorted[0].url,
      file,
      file.type || undefined,
      control,
      (loaded) => onProgress({ bytesSent: loaded, totalBytes: file.size }),
    );
    onProgress({ bytesSent: file.size, totalBytes: file.size });
    return;
  }

  const chunks = sliceFile(file, sorted);
  let sent = 0;
  for (let i = 0; i < sorted.length; i++) {
    control.throwIfStopped();
    const chunk = chunks[i];
    const base = sent;
    await putOneWithRetry(sorted[i].url, chunk, undefined, control, (loaded) => {
      onProgress({ bytesSent: base + loaded, totalBytes: file.size });
    });
    sent += chunk.size;
    onProgress({ bytesSent: sent, totalBytes: file.size });
  }
}

export { TransferAbortedError, TransferControl, TransferPausedError };
