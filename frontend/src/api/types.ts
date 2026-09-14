export type UploadMode = "single" | "multipart";

export type PresignedPart = {
  part_number: number;
  url: string;
};

export type CreateUploadResponse = {
  file_id: string;
  object_key: string;
  mode: UploadMode;
  expires_in_seconds: number;
  upload_id: string | null;
  parts: PresignedPart[];
};

export type CompleteUploadResponse = {
  file_id: string;
  status: string;
  object_key: string;
};

export type AbortUploadResponse = {
  file_id: string;
  status: string;
};

export type ApiErrorBody = {
  error?: string;
  message?: string;
};
