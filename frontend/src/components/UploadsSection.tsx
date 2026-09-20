import type { CatalogItem } from "../api/types";
import { CatalogPanel } from "./CatalogPanel";

type Props = {
  onSelect?: (fileId: string) => void;
};

export function UploadsSection({ onSelect }: Props) {
  return (
    <CatalogPanel
      kind="videos"
      headingId="library-heading"
      title="My videos"
      subtitle="Everything you have uploaded, including videos still encoding."
      empty="No uploads yet. Use the Upload tab to add a video."
      onSelect={
        onSelect ? (item: CatalogItem) => onSelect(item.file_id) : undefined
      }
    />
  );
}
