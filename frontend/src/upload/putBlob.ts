import {
  TransferAbortedError,
  TransferControl,
  TransferPausedError,
} from "./transferControl";

/** PUT one blob to a presigned URL; honors pause/abort via TransferControl. */
export async function putBlob(
  url: string,
  blob: Blob,
  contentType: string | undefined,
  control: TransferControl,
  onProgress?: (loaded: number) => void,
): Promise<void> {
  control.throwIfStopped();
  await control.waitWhilePaused();

  await new Promise<void>((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    control.currentXhr = xhr;
    xhr.open("PUT", url);
    if (contentType) xhr.setRequestHeader("Content-Type", contentType);

    xhr.upload.onprogress = (ev) => {
      if (ev.lengthComputable && onProgress) onProgress(ev.loaded);
    };
    xhr.onload = () => {
      control.currentXhr = null;
      if (xhr.status >= 200 && xhr.status < 300) resolve();
      else reject(new Error(`S3 PUT failed (${xhr.status})`));
    };
    xhr.onerror = () => {
      control.currentXhr = null;
      reject(new Error("S3 PUT network error"));
    };
    xhr.onabort = () => {
      control.currentXhr = null;
      if (control.aborted) reject(new TransferAbortedError());
      else if (control.paused) reject(new TransferPausedError());
      else reject(new TransferAbortedError());
    };
    xhr.send(blob);
  });
}
