import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { toolchainDetect } from "../lib/ipc";
import { useUi } from "../state/ui";
import { SystemLogsPanel } from "../components/SystemLogsPanel";

export function Dashboard() {
  const { t } = useTranslation();
  const setPage = useUi((s) => s.setPage);
  const [logsOpen, setLogsOpen] = useState(false);
  const { data } = useQuery({
    queryKey: ["toolchain", "summary"],
    queryFn: toolchainDetect,
    staleTime: 60_000,
  });

  const required = data
    ? [data.java, data.aapt2, data.zipalign, data.apksigner, data.adb]
    : [];
  const missing = required.filter((tool) => !tool?.found).length ?? 0;

  return (
    <div className="p-6">
      <div className="flex items-start">
        <div>
          <h1 className="text-xl font-semibold">{t("dashboard.title")}</h1>
          <p className="mt-1 text-sm text-text-dim">{t("dashboard.subtitle")}</p>
        </div>
        <button
          type="button"
          onClick={() => setLogsOpen(true)}
          className="ml-auto rounded-md border border-border bg-surface px-3 py-1.5 text-sm text-text-dim hover:bg-surface-2 hover:text-text"
        >
          {t("logs.title")}
        </button>
      </div>

      {data && (
        <div
          className={`mt-4 rounded-lg border p-4 ${
            missing === 0
              ? "border-success/30 bg-success/5"
              : "border-warning/30 bg-warning/5"
          }`}
        >
          <div className="flex items-center gap-2">
            <span
              className={`inline-block h-2.5 w-2.5 rounded-full ${
                missing === 0 ? "bg-success" : "bg-warning"
              }`}
            />
            <span className="font-medium">
              {missing === 0 ? t("dashboard.systemOk") : t("dashboard.systemMissing")}
            </span>
            <button
              type="button"
              onClick={() => setPage("system")}
              className="ml-auto rounded-md border border-border px-2 py-1 text-xs hover:bg-surface-2"
            >
              {t("system.title")}
            </button>
          </div>
          <div className="mt-3 grid grid-cols-2 gap-2 text-xs sm:grid-cols-3">
            {required.map((tool) => (
              <div key={tool?.id} className="flex items-center gap-1.5">
                <span
                  className={`inline-block h-1.5 w-1.5 rounded-full ${
                    tool?.found ? "bg-success" : "bg-danger"
                  }`}
                />
                <span className="mono text-text-dim">{tool?.id}</span>
                {tool?.found && tool.version && (
                  <span className="text-text-dim/60">{tool.version}</span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {logsOpen && <SystemLogsPanel onClose={() => setLogsOpen(false)} />}
    </div>
  );
}
