/** Mutable control plane for in-flight S3 PUTs (pause / resume / cancel). */

export class TransferAbortedError extends Error {
  constructor() {
    super("Upload aborted");
    this.name = "TransferAbortedError";
  }
}

export class TransferPausedError extends Error {
  constructor() {
    super("Upload paused");
    this.name = "TransferPausedError";
  }
}

export class TransferControl {
  aborted = false;
  paused = false;
  currentXhr: XMLHttpRequest | null = null;
  private resumeWaiters: Array<() => void> = [];

  pause(): void {
    if (this.aborted || this.paused) return;
    this.paused = true;
    this.currentXhr?.abort();
  }

  resume(): void {
    if (this.aborted || !this.paused) return;
    this.paused = false;
    const waiters = this.resumeWaiters.splice(0);
    for (const wake of waiters) wake();
  }

  abort(): void {
    this.aborted = true;
    this.paused = false;
    this.currentXhr?.abort();
    const waiters = this.resumeWaiters.splice(0);
    for (const wake of waiters) wake();
  }

  async waitWhilePaused(): Promise<void> {
    while (this.paused && !this.aborted) {
      await new Promise<void>((resolve) => {
        this.resumeWaiters.push(resolve);
      });
    }
    if (this.aborted) throw new TransferAbortedError();
  }

  throwIfStopped(): void {
    if (this.aborted) throw new TransferAbortedError();
  }
}
