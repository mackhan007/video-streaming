import type { ApiErrorBody } from "./types";

const baseUrl = () => (import.meta.env.VITE_API_BASE_URL ?? "").replace(/\/$/, "");

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export async function apiJson<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(`${baseUrl()}${path}`, {
    ...init,
    headers: {
      Accept: "application/json",
      ...(init?.body ? { "Content-Type": "application/json" } : {}),
      ...init?.headers,
    },
  });

  if (!res.ok) {
    let detail = res.statusText;
    try {
      const body = (await res.json()) as ApiErrorBody;
      detail = body.message ?? body.error ?? detail;
    } catch {
      /* keep statusText */
    }
    throw new ApiError(detail || `HTTP ${res.status}`, res.status);
  }

  if (res.status === 204) {
    return undefined as T;
  }
  return (await res.json()) as T;
}
