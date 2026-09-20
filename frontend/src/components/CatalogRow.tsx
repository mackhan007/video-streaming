import type { KeyboardEvent, ReactNode } from "react";
import type { CatalogItem } from "../api/types";
import { formatBytes } from "../ui/formatBytes";
import { formatWhen, statusChipClass } from "../ui/statusChip";

type Props = {
  item: CatalogItem;
  action?: ReactNode;
  onSelect?: () => void;
};

export function CatalogRow({ item, action, onSelect }: Props) {
  const title = item.title?.trim() || "Untitled video";
  return (
    <li
      className={`flex flex-wrap items-center gap-3 border-b border-line px-4 py-3.5 last:border-b-0 sm:px-5 ${
        onSelect ? "cursor-pointer hover:bg-paper-deep/40" : ""
      }`}
      {...(onSelect
        ? {
            role: "button",
            tabIndex: 0,
            onClick: onSelect,
            onKeyDown: (e: KeyboardEvent) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onSelect();
              }
            },
          }
        : {})}
    >
      <span
        className="flex h-10 w-10 shrink-0 items-center justify-center rounded-yt-sm bg-paper-deep text-ink-muted"
        aria-hidden
      >
        <svg viewBox="0 0 24 24" className="h-4 w-4 fill-current">
          <path d="M8 5v14l11-7z" />
        </svg>
      </span>
      <div className="min-w-0 flex-1">
        <p className="m-0 truncate text-[15px] font-semibold text-ink">{title}</p>
        <p className="m-0 mt-0.5 truncate font-mono text-[12px] text-ink-soft">
          {item.file_id}
        </p>
        <p className="m-0 mt-1 text-[12px] text-ink-muted">
          {formatBytes(item.file_size)} · {formatWhen(item.created_at)}
        </p>
      </div>
      <span
        className={`rounded-yt-sm px-2 py-1 text-[11px] font-semibold uppercase tracking-wide ${statusChipClass(item.status)}`}
      >
        {item.status}
      </span>
      {action ? (
        <span onClick={(e) => e.stopPropagation()}>{action}</span>
      ) : null}
    </li>
  );
}
