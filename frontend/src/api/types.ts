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
  started_at: string | null;
  finished_at: string | null;
  progress_pct?: number | null;
  chunks_done?: number | null;
  chunks_total?: number | null;
};

export type PipelineResponse = {
  file_id: string;
  steps: PipelineStep[];
};

export type CatalogItem = {
  file_id: string;
  title: string | null;
  status: string;
  file_size: number;
  created_at: string;
  updated_at: string;
  playback_path?: string | null;
  master_playlist_url?: string | null;
};

export type CatalogPage = {
  items: CatalogItem[];
  next_seen: string | null;
};

export type ApiErrorBody = {
  error?: string;
  message?: string;
};

export type WatchStateResponse = {
  file_id: string;
  viewer_id: string;
  position_secs: number;
  duration_secs: number | null;
};
