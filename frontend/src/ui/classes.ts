/** Shared Tailwind class strings — YouTube-studio inspired. */

export const fieldLabel =
  "text-[13px] font-semibold text-ink-muted";

export const fieldStack = "mb-5 grid gap-2";

export const inputBase =
  "w-full rounded-yt-sm border border-line bg-surface px-3.5 py-3 text-[15px] text-ink outline-none transition placeholder:text-ink-soft focus:border-ink focus:ring-1 focus:ring-ink/10 disabled:bg-paper-deep";

export const btnBase =
  "inline-flex cursor-pointer items-center justify-center gap-2 rounded-yt-sm px-5 py-2.5 text-[14px] font-semibold transition duration-150 disabled:cursor-not-allowed";

export const btnPrimary =
  `${btnBase} bg-accent text-white hover:enabled:bg-accent-hover active:enabled:scale-[0.98]`;

export const btnGhost =
  `${btnBase} border border-line bg-surface text-ink hover:enabled:bg-paper-deep`;

export const btnDanger =
  `${btnBase} border border-danger/30 bg-danger-soft text-danger hover:enabled:bg-danger/10`;

export const bannerBase =
  "mt-5 flex items-start gap-3 rounded-yt-sm px-4 py-3.5 text-[14px]";

export const bannerOk = `${bannerBase} bg-ok-soft text-ok`;
export const bannerErr = `${bannerBase} bg-danger-soft text-danger`;
export const bannerNeutral = `${bannerBase} bg-paper-deep text-ink-muted`;
export const bannerWarn = `${bannerBase} bg-warn-soft text-warn`;
