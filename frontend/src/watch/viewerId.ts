const KEY = "macky.viewer_id";

/** Stable anonymous viewer id for watch-progress Redis keys. */
export function viewerId(): string {
  const existing = localStorage.getItem(KEY);
  if (existing) return existing;
  const id = crypto.randomUUID();
  localStorage.setItem(KEY, id);
  return id;
}
