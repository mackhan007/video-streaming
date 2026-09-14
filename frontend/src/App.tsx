import { useState } from "react";
import { DeskNav, type DeskSection } from "./components/DeskNav";
import { LinksSection } from "./components/LinksSection";
import { TopBar } from "./components/TopBar";
import { UploadWorkbench } from "./components/UploadWorkbench";
import { UploadsSection } from "./components/UploadsSection";

export function App() {
  const [section, setSection] = useState<DeskSection>("upload");

  return (
    <div className="min-h-screen bg-paper">
      <TopBar brand="Macky" />
      <DeskNav current={section} onChange={setSection} />
      <main>
        {section === "upload" ? <UploadWorkbench /> : null}
        {section === "library" ? <UploadsSection /> : null}
        {section === "links" ? <LinksSection /> : null}
      </main>
    </div>
  );
}
