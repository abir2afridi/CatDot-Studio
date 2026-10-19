import { useTranslation } from "react-i18next";
import { useUi, type Page } from "../state/ui";

// Monochrome 16x16 inline SVG icons (billboard material set uses text).
const icons: Record<Page, React.ReactNode> = {
  guided: (
    <path d="M8 1.5 1 4.8l7 3.3 7-3.3L8 1.5ZM3 7.4v3.3c0 .7 2.2 2.3 5 2.3s5-1.6 5-2.3V7.4l-5 2.4-5-2.4Zm10.5 1V12h-1.6V8.6l1.6-.6Z" />
  ),
  explorer: (
    <path d="M10.5 2H2v12h12V5.5L10.5 2ZM8 3v3h5v1H8V3Zm5 10H3V3h3v5h7v5Z" />
  ),
  ai: (
    <path d="M8 1l1.8 4.2L14 7l-4.2 1.8L8 13l-1.8-4.2L2 7l4.2-1.8L8 1Z" />
  ),
  devices: (
    <path d="M5.2 1h5.6c.66 0 1.2.54 1.2 1.2v11.6c0 .66-.54 1.2-1.2 1.2H5.2A1.2 1.2 0 0 1 4 13.8V2.2C4 1.54 4.54 1 5.2 1Zm.3 2v9h5V3h-5Zm2.5 10.1a.75.75 0 1 0 .01 1.5.75.75 0 0 0-.01-1.5Z" />
  ),
  system: (
    <path d="M8 3a5 5 0 1 0 .01 10.01A5 5 0 0 0 8 3Zm0 1.6a3.4 3.4 0 1 1-.01 6.8A3.4 3.4 0 0 1 8 4.6ZM7.2 0h1.6v2H7.2V0Zm0 14h1.6v2H7.2v-2ZM0 7.2h2v1.6H0V7.2Zm14 0h2v1.6h-2V7.2Z" />
  ),
  settings: (
    <path d="M13.9 8.6a6 6 0 0 0 0-1.2l2-1.5-1.6-2.8-2.4.9a6 6 0 0 0-1-.6L10.5 0h-3l-.4 2.4a6 6 0 0 0-1 .6L3.7 2.1 2.1 4.9l2 1.5a6 6 0 0 0 0 1.2l-2 1.5 1.6 2.8 2.4-.9c.3.24.64.43 1 .6l.4 2.4h3l.4-2.4a6 6 0 0 0 1-.6l2.4.9 1.6-2.8-2-1.5ZM9 10.5A2.5 2.5 0 1 1 9 5.5a2.5 2.5 0 0 1 0 5Z" />
  ),
};

interface Item {
  id: Page;
  labelKey: string;
}

const items: Item[] = [
  { id: "guided", labelKey: "nav.guided" },
  { id: "explorer", labelKey: "nav.explorer" },
  { id: "ai", labelKey: "nav.ai" },
  { id: "devices", labelKey: "nav.devices" },
  { id: "system", labelKey: "nav.system" },
  { id: "settings", labelKey: "nav.settings" },
];

export function ActivityBar() {
  const { t } = useTranslation();
  const page = useUi((s) => s.page);
  const setPage = useUi((s) => s.setPage);

  return (
    <nav className="flex w-14 shrink-0 flex-col items-center border-r border-border bg-surface py-2">
      {items.map((item) => {
        const active = page === item.id;
        const cls = active
          ? "border-l-2 border-accent text-text"
          : "border-l-2 border-transparent text-text-dim hover:text-text";
        return (
          <button
            key={item.id}
            type="button"
            title={t(item.labelKey)}
            onClick={() => setPage(item.id)}
            aria-label={t(item.labelKey)}
            className={`flex h-12 w-14 items-center justify-center border-r-2 border-transparent transition-colors ${cls}`}
          >
            <svg
              viewBox="0 0 16 16"
              width="20"
              height="20"
              fill="currentColor"
              aria-hidden="true"
            >
              {icons[item.id]}
            </svg>
          </button>
        );
      })}
    </nav>
  );
}
