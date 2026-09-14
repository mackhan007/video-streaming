import { useCallback, useEffect, useState } from "react";
import { listingApi, type CatalogKind } from "../api/listingApi";
import type { CatalogItem } from "../api/types";

const PAGE = 20;

/** Loads a catalog page; `loadMore` uses `next_seen`. */
export function useCatalog(kind: CatalogKind) {
  const [items, setItems] = useState<CatalogItem[]>([]);
  const [nextSeen, setNextSeen] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [tick, setTick] = useState(0);

  useEffect(() => {
    let alive = true;
    setLoading(true);
    setError(null);
    void listingApi
      .page(kind, PAGE)
      .then((page) => {
        if (!alive) return;
        setItems(page.items);
        setNextSeen(page.next_seen);
      })
      .catch((e: unknown) => {
        if (!alive) return;
        setError(e instanceof Error ? e.message : "Failed to load");
        setItems([]);
        setNextSeen(null);
      })
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, [kind, tick]);

  const loadMore = useCallback(async () => {
    if (!nextSeen || loadingMore) return;
    setLoadingMore(true);
    try {
      const page = await listingApi.page(kind, PAGE, nextSeen);
      setItems((prev) => [...prev, ...page.items]);
      setNextSeen(page.next_seen);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to load more");
    } finally {
      setLoadingMore(false);
    }
  }, [kind, nextSeen, loadingMore]);

  const refresh = useCallback(() => setTick((n) => n + 1), []);

  return { items, loading, loadingMore, error, hasMore: Boolean(nextSeen), loadMore, refresh };
}
