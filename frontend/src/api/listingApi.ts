import { apiJson } from "./client";
import type { CatalogPage } from "./types";

export type CatalogKind = "videos" | "links";

function path(kind: CatalogKind, limit: number, seen?: string | null): string {
  const q = new URLSearchParams({ limit: String(limit) });
  if (seen) q.set("seen", seen);
  return kind === "links" ? `/lister/links?${q}` : `/lister/videos?${q}`;
}

/** REST gateway for listing catalogs. */
export const listingApi = {
  page(kind: CatalogKind, limit = 20, seen?: string | null): Promise<CatalogPage> {
    return apiJson(path(kind, limit, seen));
  },
};
