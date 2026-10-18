import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { toolchainDetect } from "../lib/ipc";
import type { ToolInfo } from "@catdot/shared-types";

function ToolRow({ tool, labels }: { tool: ToolInfo; labels: { found: string; missing: string; version: string; path: string; fixHint: string } }) {
  return (
    <div className="rounded-lg border border-border bg-surface p-3">
      <div className="flex items-center gap-2">
        <span
          className={`inline-block h-2 w-2 rounded-full ${
            tool.found ? "bg-success" : "bg-danger"
          }`}
        />
        <span className="mono font-medium">{tool.id}</span>
        <span className={`ml-auto text-xs ${tool.found ? "text-success" : "text-danger"}`}>
          {tool.found ? labels.found : labels.missing}
        </span>
      </div>
      {tool.found && (
        <div className="mt-2 space-y-0.5 text-xs text-text-dim">
          {tool.version && (
            <div>
              <span className="text-text-dim/60">{labels.version}: </span>
              <span className="mono">{tool.version}</span>
            </div>
          )}
          {tool.path && (
            <div>
              <span className="text-text-dim/60">{labels.path}: </span>
              <span className="mono break-all">{tool.path}</span>
            </div>
          )}
        </div>
      )}
      {!tool.found && tool.fix_hint && (
        <div className="mt-2 rounded bg-warning/10 px-2 py-1 text-xs text-warning">
          {labels.fixHint}: {tool.fix_hint}
        </div>
      )}
    </div>
  );
}

export function SystemCheckPage() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const { data, isFetching } = useQuery({
    queryKey: ["toolchain", "detail"],
    queryFn: toolchainDetect,
    staleTime: 0,
  });
  const redetect = useMutation({
    mutationFn: toolchainDetect,
    onSuccess: (report) => qc.setQueryData(["toolchain", "detail"], report),
  });

  const labels = {
    found: t("system.found"),
    missing: t("system.missing"),
    version: t("system.version"),
    path: t("system.path"),
    fixHint: t("system.fixHint"),
  };

  const tools: ToolInfo[] = data
    ? [
        data.java,
        data.aapt2,
        data.zipalign,
        data.apksigner,
        data.adb,
        data.emulator,
        data.apktool,
        data.jadx,
      ]
    : [];

  return (
    <div className="p-6">
      <div className="flex items-start">
        <div>
          <h1 className="text-xl font-semibold">{t("system.title")}</h1>
          <p className="mt-1 text-sm text-text-dim">{t("system.subtitle")}</p>
        </div>
        <button
          type="button"
          onClick={() => redetect.mutate()}
          disabled={isFetching}
          className="ml-auto rounded-md border border-border bg-surface px-3 py-1.5 text-sm hover:bg-surface-2 disabled:opacity-50"
        >
          {isFetching ? t("common.loading") : t("system.detect")}
        </button>
      </div>

      {data && (
        <div className="mt-3 text-xs text-text-dim">
          long_paths:{" "}
          <span className={data.long_paths_enabled ? "text-success" : "text-warning"}>
            {data.long_paths_enabled ? "enabled" : "disabled"}
          </span>
        </div>
      )}

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        {tools.map((tool) => (
          <ToolRow key={tool.id} tool={tool} labels={labels} />
        ))}
      </div>
    </div>
  );
}
