import { useEffect, useState } from "react";

type Probe = { label: string; ok: boolean | null };

async function ping(url: string): Promise<boolean> {
  try {
    const res = await fetch(url, { cache: "no-store" });
    return res.ok;
  } catch {
    return false;
  }
}

/** Live health chips for EMS / IMS / CDN. */
export function ServiceHealthBar() {
  const [probes, setProbes] = useState<Probe[]>([
    { label: "EMS", ok: null },
    { label: "IMS", ok: null },
    { label: "CDN", ok: null },
  ]);

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      const [ems, ims, cdn] = await Promise.all([
        ping("/health"),
        ping("/ims-health"),
        ping("/cdn-health"),
      ]);
      if (!alive) return;
      setProbes([
        { label: "EMS", ok: ems },
        { label: "IMS", ok: ims },
        { label: "CDN", ok: cdn },
      ]);
    };
    void tick();
    const id = window.setInterval(() => void tick(), 4000);
    return () => {
      alive = false;
      window.clearInterval(id);
    };
  }, []);

  return (
    <div className="flex flex-wrap items-center gap-2">
      {probes.map((p) => (
        <span
          key={p.label}
          className={`inline-flex items-center gap-1.5 rounded-yt-sm border px-2.5 py-1 text-[12px] font-semibold ${
            p.ok === null
              ? "border-line text-ink-soft"
              : p.ok
                ? "border-ok/30 bg-ok-soft text-ok"
                : "border-danger/30 bg-danger-soft text-danger"
          }`}
        >
          <span
            className={`h-1.5 w-1.5 rounded-full ${
              p.ok === null ? "bg-ink-soft" : p.ok ? "bg-ok" : "bg-danger"
            }`}
          />
          {p.label}
        </span>
      ))}
    </div>
  );
}
