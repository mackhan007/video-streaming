import { CatalogPanel } from "./CatalogPanel";

export function UploadsSection() {
  return (
    <CatalogPanel
      kind="videos"
      headingId="library-heading"
      title="My videos"
      subtitle="Everything you have uploaded, including videos still encoding."
      empty="No uploads yet. Use the Upload tab to add a video."
    />
  );
}
