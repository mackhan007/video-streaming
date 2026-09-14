type Props = {
  brand: string;
};

export function TopBar({ brand }: Props) {
  return (
    <header className="sticky top-0 z-20 border-b border-line bg-surface/95 backdrop-blur-md">
      <div className="mx-auto flex h-14 max-w-5xl items-center gap-3 px-4 sm:px-6">
        <span
          className="flex h-8 w-8 items-center justify-center rounded-yt-sm bg-accent text-white shadow-sm"
          aria-hidden
        >
          <svg viewBox="0 0 24 24" className="h-4 w-4 fill-current" aria-hidden>
            <path d="M8 5v14l11-7z" />
          </svg>
        </span>
        <div className="min-w-0">
          <p className="truncate text-[18px] font-extrabold tracking-tight text-ink">
            {brand}
          </p>
        </div>
        <span className="ml-auto hidden text-[13px] font-medium text-ink-soft sm:inline">
          Studio
        </span>
      </div>
    </header>
  );
}
