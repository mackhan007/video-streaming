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

export type VideoStatusResponse = {
  file_id: string;
  status: string;
  title: string | null;
  playback_path: string | null;
  file_size: number;
  updated_at: string;
};

export type StreamResponse = {
  file_id: string;
  status: string;
  master_playlist_url: string;
};

export type PipelineStepState = "pending" | "running" | "done" | "failed";

export type PipelineStepName =
  | "upload"
  | "queue"
  | "process"
  | "hls_360"
  | "hls_720"
  | "hls_1080"
  | "ready"
  | "play";

export type PipelineStep = {
  step: PipelineStepName;
  state: PipelineStepState;
  detail: string | null;
  error: string | null;
};

export type PipelineResponse = {
  file_id: string;
  steps: PipelineStep[];
};

export type ApiErrorBody = {
  error?: string;
  message?: string;
};
