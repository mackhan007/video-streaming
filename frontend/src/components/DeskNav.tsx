export type DeskSection = "upload" | "library" | "links";

const TABS: { id: DeskSection; label: string }[] = [
  { id: "upload", label: "Upload" },
  { id: "library", label: "My videos" },
  { id: "links", label: "Watch" },
];

type Props = {
  current: DeskSection;
  onChange: (section: DeskSection) => void;
};

export function DeskNav({ current, onChange }: Props) {
  return (
    <nav
      className="border-b border-line bg-surface"
      aria-label="Desk sections"
    >
      <div className="mx-auto flex max-w-5xl gap-1 px-4 sm:px-6">
        {TABS.map((tab) => {
          const active = tab.id === current;
          return (
            <button
              key={tab.id}
              type="button"
              onClick={() => onChange(tab.id)}
              aria-current={active ? "page" : undefined}
              className={`relative -mb-px cursor-pointer border-b-2 px-3 py-3 text-[14px] font-semibold transition ${
                active
                  ? "border-ink text-ink"
                  : "border-transparent text-ink-muted hover:text-ink"
              }`}
            >
              {tab.label}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
