import { TopBar } from "./components/TopBar";
import { UploadWorkbench } from "./components/UploadWorkbench";

export function App() {
  return (
    <div className="min-h-screen bg-paper">
      <TopBar brand="Macky" />
      <main>
        <UploadWorkbench />
      </main>
    </div>
  );
}
