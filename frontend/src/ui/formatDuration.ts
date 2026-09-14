/** Format a millisecond duration for pipeline step cards. */
export function formatDuration(ms: number): string {
  if (ms < 0) ms = 0;
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const rem = s % 60;
  if (m < 60) return rem ? `${m}m ${rem}s` : `${m}m`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm ? `${h}h ${rm}m` : `${h}h`;
}

/** Elapsed ms for a step; live `now` while running. */
export function stepDurationMs(
  startedAt: string | null,
  finishedAt: string | null,
  running: boolean,
  now: number,
): number | null {
  if (!startedAt) return null;
  const start = Date.parse(startedAt);
  if (Number.isNaN(start)) return null;
  if (finishedAt) {
    const end = Date.parse(finishedAt);
    if (Number.isNaN(end)) return null;
    return end - start;
  }
  if (running) return now - start;
  return null;
}
