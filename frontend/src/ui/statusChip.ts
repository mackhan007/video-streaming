export function statusChipClass(status: string): string {
  switch (status) {
    case "ready":
      return "bg-ok-soft text-ok";
    case "failed":
      return "bg-danger-soft text-danger";
    case "processing":
    case "uploaded":
      return "bg-warn-soft text-warn";
    default:
      return "bg-paper-deep text-ink-muted";
  }
}

export function formatWhen(iso: string): string {
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return iso;
  return new Date(t).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
