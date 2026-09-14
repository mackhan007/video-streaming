import { apiJson } from "./client";
import type {
  AbortUploadResponse,
  CompleteUploadResponse,
  CreateUploadResponse,
  PipelineResponse,
  StreamResponse,
  VideoStatusResponse,
} from "./types";

export type CreateUploadInput = {
  fileSize: number;
  title?: string;
  contentType?: string;
};

/** REST gateway for upload + stream. */
export const uploadApi = {
  create(input: CreateUploadInput): Promise<CreateUploadResponse> {
    return apiJson("/uploader/videos", {
      method: "POST",
      body: JSON.stringify({
        file_size: input.fileSize,
        title: input.title || undefined,
        content_type: input.contentType || undefined,
      }),
    });
  },

  complete(fileId: string): Promise<CompleteUploadResponse> {
    return apiJson(`/uploader/videos/${fileId}/complete`, { method: "POST" });
  },

  abort(fileId: string): Promise<AbortUploadResponse> {
    return apiJson(`/uploader/videos/${fileId}/abort`, { method: "POST" });
  },

  status(fileId: string): Promise<VideoStatusResponse> {
    return apiJson(`/uploader/videos/${fileId}`);
  },

  pipeline(fileId: string): Promise<PipelineResponse> {
    return apiJson(`/uploader/videos/${fileId}/pipeline`);
  },

  retry(fileId: string): Promise<{ file_id: string; status: string }> {
    return apiJson(`/uploader/videos/${fileId}/retry`, { method: "POST" });
  },

  stream(fileId: string): Promise<StreamResponse> {
    return apiJson(`/streamer/stream?file_id=${encodeURIComponent(fileId)}`);
  },
};
