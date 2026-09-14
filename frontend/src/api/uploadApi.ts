import { apiJson } from "./client";
import type {
  AbortUploadResponse,
  CompleteUploadResponse,
  CreateUploadResponse,
} from "./types";

export type CreateUploadInput = {
  fileSize: number;
  title?: string;
  contentType?: string;
};

/** REST gateway for `/uploader/videos`. */
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
};
