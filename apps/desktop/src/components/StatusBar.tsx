import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { toolchainDetect } from "../lib/ipc";
import { apkStatus } from "../lib/apk";

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

  const required = data
    ? [data.java, data.aapt2, data.zipalign, data.apksigner, data.adb]
    : [];
  const missing = required.filter((tool) => !tool?.found).length;
  const ready = data != null && missing === 0;

  return (
    <footer className="flex items-center gap-4 border-t border-border bg-surface px-4 py-1.5 text-xs text-text-dim">
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
        {apk.data?.imported ? (
          <>
            <span className="text-success">{t("status.imported")}</span>
            {apk.data.pending_edits > 0 && (
              <span className="text-warning">
                {t("status.pendingEdits", { count: apk.data.pending_edits })}
              </span>
            )}
            {apk.data.signed_apk != null && (
              <span className="text-success">✓ {t("status.signed")}</span>
            )}
          </>
        ) : (
          <span>{t("status.notImported")}</span>
        )}
        <span>{t("status.phase")}</span>
      </span>
    </footer>
  );
}
