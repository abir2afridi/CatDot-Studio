// Command palette (Part X1): Ctrl+Shift+P, fuzzy filter over the command
// registry; disabled commands stay visible with their honest reason.
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { filterCommands } from "../lib/commands-filter";
import { useCommands } from "../lib/commands";
import { useUi } from "../state/ui";

export function CommandPalette() {
  const { t } = useTranslation();
  const open = useUi((s) => s.paletteOpen);
  const setOpen = useUi((s) => s.setPaletteOpen);
  const selected = useUi((s) => s.paletteSelected);
  const setSelected = useUi((s) => s.setPaletteSelected);
  const commands = useCommands();
  const [query, setQuery] = useState("");

  useEffect(() => {
    if (open) setQuery("");
  }, [open]);

  const results = useMemo(
    () => filterCommands(commands, query),
    [commands, query],
  );

  useEffect(() => {
    if (selected >= results.length) setSelected(Math.max(0, results.length - 1));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [results.length]);

  if (!open) return null;

  const run = (i: number) => {
    const c = results[i];
    if (c != null && c.disabledReason == null) {
      setOpen(false);
      c.run();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/40 pt-16"
      onClick={() => setOpen(false)}
      role="presentation"
    >
      <div
        className="w-[560px] max-w-[90vw] rounded-lg border border-border bg-surface shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label={t("palette.title")}
      >
        <div className="p-2">
          <input
            autoFocus
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelected(0);
            }}
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") {
                setSelected(Math.min(selected + 1, results.length - 1));
              } else if (e.key === "ArrowUp") {
                setSelected(Math.max(selected - 1, 0));
              } else if (e.key === "Enter") {
                run(selected);
              } else if (e.key === "Escape") {
                setOpen(false);
              }
            }}
            placeholder={t("palette.placeholder")}
            aria-label={t("palette.placeholder")}
            className="w-full rounded-md bg-bg px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-accent"
          />
        </div>
        <ul
          className="max-h-[420px] overflow-y-auto border-t border-border py-1"
          role="listbox"
          aria-label={t("palette.title")}
        >
          {results.length === 0 && (
            <li className="px-4 py-3 text-sm text-text-dim">{t("palette.noResults")}</li>
          )}
          {results.map((c, i) => (
            <li key={c.id} role="option" aria-selected={i === selected}>
              <button
                type="button"
                disabled={c.disabledReason != null}
                onMouseEnter={() => setSelected(i)}
                onClick={() => run(i)}
                title={c.disabledReason ?? c.titleKey}
                className={`flex w-full items-center gap-2 px-4 py-1.5 text-left text-sm ${
                  i === selected ? "bg-accent/15 text-text" : "text-text-dim"
                } ${c.disabledReason != null ? "cursor-not-allowed opacity-40" : ""}`}
              >
                <span className="truncate">{c.title}</span>
                <span className="ml-auto shrink-0 text-[10px] uppercase text-text-dim/70">
                  {c.disabledReason ?? c.id}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
