import type { ReactNode } from "react";
import type { CatalogItem } from "../api/types";
import { useCatalog } from "../hooks/useCatalog";
import { btnGhost, btnPrimary } from "../ui/classes";
import { CatalogRow } from "./CatalogRow";

type Props = {
  title: string;
  subtitle: string;
  kind: "videos" | "links";
  headingId: string;
  empty: string;
  action?: (item: CatalogItem) => ReactNode;
  onSelect?: (item: CatalogItem) => void;
  footer?: ReactNode;
};

export function CatalogPanel({
  title,
  subtitle,
  kind,
  headingId,
  empty,
  action,
  onSelect,
  footer,
}: Props) {
  const { items, loading, loadingMore, error, hasMore, loadMore, refresh } =
    useCatalog(kind);

  return (
    <section
      className="animate-rise-delay mx-auto w-full max-w-3xl px-4 pb-16 pt-8 sm:px-6"
      aria-labelledby={headingId}
    >
      <div className="mb-6 flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1
            id={headingId}
            className="m-0 text-[28px] font-extrabold tracking-tight text-ink sm:text-[32px]"
          >
            {title}
          </h1>
          <p className="mt-2 max-w-xl text-[15px] text-ink-muted">{subtitle}</p>
        </div>
        <button type="button" className={btnGhost} onClick={refresh} disabled={loading}>
          Refresh
        </button>
      </div>

      <div className="overflow-hidden rounded-yt bg-surface shadow-yt">
        {loading ? (
          <p className="m-0 px-5 py-10 text-center text-[14px] text-ink-muted">
            Loading…
          </p>
        ) : error ? (
          <p className="m-0 px-5 py-10 text-center text-[14px] text-danger">{error}</p>
        ) : items.length === 0 ? (
          <p className="m-0 px-5 py-10 text-center text-[14px] text-ink-muted">{empty}</p>
        ) : (
          <ul className="m-0 list-none p-0">
            {items.map((item) => (
              <CatalogRow
                key={item.file_id}
                item={item}
                action={action?.(item)}
                onSelect={onSelect ? () => onSelect(item) : undefined}
              />
            ))}
          </ul>
        )}
        {hasMore ? (
          <div className="border-t border-line px-5 py-3">
            <button
              type="button"
              className={btnPrimary}
              disabled={loadingMore}
              onClick={() => void loadMore()}
            >
              {loadingMore ? "Loading…" : "Load more"}
            </button>
          </div>
        ) : null}
      </div>
      {footer}
    </section>
  );
}
