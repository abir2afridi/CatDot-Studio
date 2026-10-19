// Device logs tab (Phase 4): a bounded, manual-refresh `adb logcat -d -t N`
// dump. Real device output only — when no device/adb exists the structured
// error text is shown honestly (never a fake empty log).
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { deviceLogcat } from "../lib/apk";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";

const LINE_OPTIONS = [100, 300, 1000] as const;

export function DeviceLogsTab() {
  const { t } = useTranslation();
  const serial = useUi((s) => s.deviceSerial);
  const [lines, setLines] = useState<number>(300);
  const [nonce, setNonce] = useState(0);
  const q = useQuery({
    queryKey: ["device", "logcat", serial, lines, nonce],
    queryFn: () => deviceLogcat(lines, serial),
    retry: false,
    staleTime: 0,
  });

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border px-2 py-1 text-[11px]">
        <button
          type="button"
          onClick={() => setNonce((n) => n + 1)}
          disabled={q.isFetching}
          className="rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text disabled:opacity-40"
        >
          {q.isFetching ? t("device.reading") : t("device.refresh")}
        </button>
        <label className="flex items-center gap-1 text-text-dim">
          {t("device.lines")}
          <select
            value={lines}
            onChange={(e) => setLines(Number(e.target.value))}
            className="rounded bg-bg px-1 py-0.5"
          >
            {LINE_OPTIONS.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <span className="truncate text-text-dim/60" title={serial ?? undefined}>
          {serial != null ? t("devices.target", { serial }) : t("devices.targetAuto")}
        </span>
        {q.data != null && (
          <span className="text-text-dim/60">
            {t("device.lineCount", { count: q.data.length === 0 ? 0 : q.data.split("\n").length })}
          </span>
        )}
        {q.data != null && q.data !== "" && (
          <button
            type="button"
            onClick={() => void navigator.clipboard.writeText(q.data).catch(() => {})}
            className="ml-auto rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text"
          >
            {t("logs.copyAll")}
          </button>
        )}
      </div>
      <div className="min-h-0 flex-1 overflow-auto p-2">
        {q.isError ? (
          <p className="text-xs text-warning">{describe(q.error)}</p>
        ) : q.data == null ? (
          <p className="text-xs text-text-dim">{t("device.pressRefresh")}</p>
        ) : q.data.trim() === "" ? (
          <p className="text-xs text-text-dim">{t("device.empty")}</p>
        ) : (
          <pre className="whitespace-pre-wrap break-all font-mono text-[11px] leading-4">
            {q.data}
          </pre>
        )}
      </div>
    </div>
  );
}

function describe(e: unknown): string {
  return e instanceof IpcError
    ? e.appError.user_message || e.appError.message
    : String(e);
}
