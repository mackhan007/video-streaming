import { useEffect, useRef } from "react";
import { streamApi } from "../api/streamApi";

const SAVE_EVERY_MS = 5000;
const RESTART_TAIL_SECS = 3;

function shouldResume(position: number, duration: number | null): boolean {
  if (position < 1) return false;
  if (duration && duration > 0 && position >= duration - RESTART_TAIL_SECS) {
    return false;
  }
  return true;
}

/** Load Redis watch progress and persist currentTime (throttled). */
export function useWatchProgress(
  fileId: string | undefined,
  video: HTMLVideoElement | null,
) {
  const lastSave = useRef(0);
  const restored = useRef(false);

  useEffect(() => {
    restored.current = false;
    lastSave.current = 0;
    if (!fileId || !video) return;

    let alive = true;
    void streamApi.getUserState(fileId).then((state) => {
      if (!alive) return;
      if (!state) {
        restored.current = true;
        return;
      }
      const seek = () => {
        if (restored.current && video.currentTime >= 1) return;
        const dur = Number.isFinite(video.duration) ? video.duration : state.duration_secs;
        if (!shouldResume(state.position_secs, dur)) {
          restored.current = true;
          return;
        }
        video.currentTime = state.position_secs;
        restored.current = true;
      };
      if (video.readyState >= 1) seek();
      else video.addEventListener("loadedmetadata", seek, { once: true });
    }).catch(() => {
      if (alive) restored.current = true;
    });

    const persist = (force = false) => {
      if (!restored.current) return;
      const now = Date.now();
      if (!force && now - lastSave.current < SAVE_EVERY_MS) return;
      lastSave.current = now;
      const pos = video.currentTime;
      const dur = Number.isFinite(video.duration) ? video.duration : undefined;
      void streamApi.saveUserState({
        fileId,
        positionSecs: pos,
        durationSecs: dur,
      });
    };

    const onTick = () => persist(false);
    video.addEventListener("timeupdate", onTick);
    video.addEventListener("pause", onTick);
    video.addEventListener("ended", onTick);
    return () => {
      alive = false;
      video.removeEventListener("timeupdate", onTick);
      video.removeEventListener("pause", onTick);
      video.removeEventListener("ended", onTick);
      persist(true);
    };
  }, [fileId, video]);
}
