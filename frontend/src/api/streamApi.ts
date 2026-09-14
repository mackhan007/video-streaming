import { apiJson, ApiError } from "./client";
import type { WatchStateResponse } from "./types";
import { viewerId } from "../watch/viewerId";

export type SaveWatchInput = {
  fileId: string;
  positionSecs: number;
  durationSecs?: number | null;
};

/** REST gateway for streamer watch progress. */
export const streamApi = {
  saveUserState(input: SaveWatchInput): Promise<WatchStateResponse> {
    return apiJson("/streamer/save-user-state", {
      method: "POST",
      body: JSON.stringify({
        file_id: input.fileId,
        viewer_id: viewerId(),
        position_secs: input.positionSecs,
        duration_secs: input.durationSecs ?? undefined,
      }),
    });
  },

  async getUserState(fileId: string): Promise<WatchStateResponse | null> {
    const q = new URLSearchParams({
      file_id: fileId,
      viewer_id: viewerId(),
    });
    try {
      return await apiJson(`/streamer/user-state?${q}`);
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) return null;
      throw e;
    }
  },
};
