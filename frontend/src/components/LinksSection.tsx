import { useState } from "react";
import type { CatalogItem } from "../api/types";
import { btnGhost, btnPrimary } from "../ui/classes";
import { CatalogPanel } from "./CatalogPanel";
import { VideoPlayer } from "./VideoPlayer";

export function LinksSection() {
  const [playing, setPlaying] = useState<CatalogItem | null>(null);

  return (
    <CatalogPanel
      kind="links"
      headingId="links-heading"
      title="Watch"
      subtitle="Ready videos with playback links. Open a playlist or play it here."
      empty="No ready videos yet. Upload a file and wait until IMS marks it ready."
      action={(item) => (
        <div className="flex flex-wrap gap-2">
          {item.master_playlist_url ? (
            <>
              <button
                type="button"
                className={btnPrimary}
                onClick={() => setPlaying(item)}
              >
                Play
              </button>
              <a
                className={`${btnGhost} no-underline`}
                href={item.master_playlist_url}
                target="_blank"
                rel="noreferrer"
              >
                Link
              </a>
            </>
          ) : null}
        </div>
      )}
      footer={
        playing?.master_playlist_url ? (
          <VideoPlayer
            src={playing.master_playlist_url}
            title={playing.title}
            fileId={playing.file_id}
          />
        ) : null
      }
    />
  );
}
