import { useEffect, useRef, useState } from "react";
import Hls from "hls.js";
import { useWatchProgress } from "../hooks/useWatchProgress";

type Props = {
  src: string;
  title?: string | null;
  fileId?: string;
};

/** HLS player (native Safari + hls.js elsewhere). */
export function VideoPlayer({ src, title, fileId }: Props) {
  const ref = useRef<HTMLVideoElement>(null);
  const [el, setEl] = useState<HTMLVideoElement | null>(null);
  useWatchProgress(fileId, el);

  useEffect(() => {
    const video = ref.current;
    setEl(video);
    if (!video || !src) return;

    if (video.canPlayType("application/vnd.apple.mpegurl")) {
      video.src = src;
      return;
    }

    if (Hls.isSupported()) {
      const hls = new Hls({ enableWorker: true });
      hls.loadSource(src);
      hls.attachMedia(video);
      return () => hls.destroy();
    }

    video.src = src;
  }, [src]);

  return (
    <div className="mt-6 overflow-hidden rounded-yt bg-ink shadow-yt-lg">
      <video
        ref={ref}
        className="aspect-video w-full bg-black"
        controls
        playsInline
        poster=""
      />
      <div className="border-t border-white/10 px-4 py-3 text-[13px] text-white/80">
        {title || "Playback"} ·{" "}
        <a className="text-white underline" href={src} target="_blank" rel="noreferrer">
          open playlist
        </a>
      </div>
    </div>
  );
}
