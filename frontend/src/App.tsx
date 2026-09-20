import { useState } from "react";
import { DeskNav, type DeskSection } from "./components/DeskNav";
import { LinksSection } from "./components/LinksSection";
import { TopBar } from "./components/TopBar";
import { UploadWorkbench } from "./components/UploadWorkbench";
import { UploadsSection } from "./components/UploadsSection";

export function App() {
  const [section, setSection] = useState<DeskSection>("upload");
  const [viewFileId, setViewFileId] = useState<string | null>(null);

  return (
    <div className="min-h-screen bg-paper">
      <TopBar brand="Macky" />
      <DeskNav current={section} onChange={setSection} />
      <main>
        {section === "upload" ? <UploadWorkbench viewFileId={viewFileId} /> : null}
        {section === "library" ? (
          <UploadsSection
            onSelect={(fileId) => {
              setViewFileId(fileId);
              setSection("upload");
            }}
          />
        ) : null}
        {section === "links" ? <LinksSection /> : null}
      </main>
    </div>
  );
}
