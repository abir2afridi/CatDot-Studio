// Status bar (Part X1): toolchain readiness, APK state, cursor position,
// panel toggles, palette entry, and one-shot mutation notices. Real feeds
// only — a green state appears only after the real operation succeeded.
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { toolchainDetect } from "../lib/ipc";
import { apkStatus } from "../lib/apk";
import { useUi } from "../state/ui";

export function StatusBar() {
  const { t } = useTranslation();
  const { data } = useQuery({
    queryKey: ["toolchain", "summary"],
    queryFn: toolchainDetect,
    staleTime: 60_000,
  });
  const apk = useQuery({
    queryKey: ["apk", "status"],
    queryFn: apkStatus,
    staleTime: 5_000,
  });

  const cursorLine = useUi((s) => s.cursorLine);
  const cursorCol = useUi((s) => s.cursorCol);
  const activeFile = useUi((s) => s.activeFile);
  const notice = useUi((s) => s.notice);
  const setNotice = useUi((s) => s.setNotice);
  const bottomOpen = useUi((s) => s.bottomOpen);
  const toggleBottom = useUi((s) => s.toggleBottom);
  const setActiveBottomTab = useUi((s) => s.setActiveBottomTab);
  const activeBottomTab = useUi((s) => s.activeBottomTab);

  // Auto-dismiss transient notices after 6 s.
  // Remove the effect-based dismissal timer; the CSS animation below fades
  // the notice out, and onAnimationEnd clears the state (no JS timers).

  const required = data ? [data.java, data.aapt2, data.zipalign, data.apksigner, data.adb] : [];
  const missing = required.filter((tool) => !tool?.found).length;
  const ready = data != null && missing === 0;

  return (
    <footer className="flex items-center gap-3 border-t border-border bg-surface px-3 py-1 text-[11px] text-text-dim">
      <span className="flex items-center gap-1.5">
        <span
          className={`inline-block h-2 w-2 rounded-full ${
            data == null ? "bg-text-dim/40" : ready ? "bg-success" : "bg-warning"
          }`}
        />
        {data == null
          ? t("common.loading")
          : ready
            ? t("dashboard.systemOk")
            : t("dashboard.systemMissing")}
      </span>
      {data != null && missing > 0 && (
        <span className="text-warning">{missing} missing</span>
      )}

      <span className="ml-auto flex items-center gap-3">
        {activeFile != null && (
          <button
            type="button"
            onClick={() => useUi.getState().setPaletteOpen(true)}
            className="hover:text-text"
            title="Ctrl+Shift+P"
          >
            Ln {cursorLine}, Col {cursorCol} · {t("palette.openStatusBar")}
          </button>
        )}
        {apk.data?.imported ? (
          <>
            <span className="text-success">{t("status.imported")}</span>
            {apk.data.pending_edits > 0 && (
              <span className="text-warning">
                {t("status.pendingEdits", { count: apk.data.pending_edits })}
              </span>
            )}
            {apk.data.signed_apk != null && (
              <span className="text-success" title={apk.data.signed_apk}>
                ✓ {t("status.signed")}
              </span>
            )}
          </>
        ) : (
          <span>{t("status.notImported")}</span>
        )}
        <span>{t("status.phase")}</span>
      </span>

      <span className="h-3.5 w-px bg-border" aria-hidden="true" />
      <button
        type="button"
        onClick={() => {
          setActiveBottomTab("logs");
          if (!bottomOpen || activeBottomTab !== "logs") {
            if (!bottomOpen) toggleBottom();
            else setActiveBottomTab("logs");
          }
        }}
        className={`hover:text-text ${bottomOpen && activeBottomTab === "logs" ? "text-text" : ""}`}
        title="Ctrl+J"
      >
        {t("panel.logs")}
      </button>
      <button
        type="button"
        onClick={() => {
          setActiveBottomTab("build");
          if (!bottomOpen || activeBottomTab !== "build") {
            if (!bottomOpen) toggleBottom();
            else setActiveBottomTab("build");
          }
        }}
        className={`hover:text-text ${bottomOpen && activeBottomTab === "build" ? "text-text" : ""}`}
        title="Ctrl+J"
      >
        {t("panel.build")}
      </button>
      <span className="h-3.5 w-px bg-border" aria-hidden="true" />
      <button
        type="button"
        onClick={() => useUi.getState().setPaletteOpen(true)}
        className="hover:text-text"
        title="Ctrl+Shift+P"
      >
        {t("palette.openStatusBar")}
      </button>
      {notice != null && (
        <span
          key={notice.text}
          role="status"
          onAnimationEnd={() => setNotice(null)}
          className={`notice-fade max-w-[360px] truncate ${notice.ok ? "text-success" : "text-danger"}`}
          title={notice.text}
        >
          {notice.text}
        </span>
      )}
    </footer>
  );
}
