// Diff tab (bottom panel): a real side-by-side diff of the active file's
// committed workspace content vs the in-memory draft (F4 review affordance).
// Requires a draft; hidden otherwise (honest empty state).
import { DiffEditor } from "@monaco-editor/react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkOpenEntry } from "../lib/apk";
import { monacoLanguage, monacoTheme } from "../lib/monaco";
import { useUi } from "../state/ui";

export function DiffTab() {
  const { t } = useTranslation();
  const entryPath = useUi((s) => s.activeFile);
  const draft = useUi((s) => (entryPath == null ? null : s.drafts[entryPath]));
  const theme = useUi((s) => s.theme);

  const entry = useQuery({
    queryKey: ["apk", "entry", entryPath],
    queryFn: () => apkOpenEntry(entryPath ?? ""),
    enabled: entryPath != null && draft != null,
    retry: false,
  });

  if (entryPath == null || draft == null) {
    return <Empty text={t("diff.noDraft")} />;
  }
  if (entry.isPending) {
    return <Empty text={t("common.loading")} />;
  }
  if (entry.isError || entry.data == null) {
    return <Empty text={t("diff.unavailable")} />;
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border px-2 py-0.5 text-[11px] text-text-dim" title={entryPath}>
        {t("diff.reviewing", { name: entryPath.split("/").pop() ?? entryPath })}
      </div>
      <div className="min-h-0 flex-1">
        <DiffEditor
          language={monacoLanguage(entryPath)}
          theme={monacoTheme(theme)}
          original={entry.data.content}
          modified={draft}
          loading={<Empty text={t("common.loading")} />}
          options={{
            readOnly: true,
            renderSideBySide: true,
            minimap: { enabled: false },
            fontSize: 13,
            fontFamily: "Cascadia Code, Consolas, Menlo, monospace",
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>
    </div>
  );
}

function Empty({ text }: { text: string }) {
  return <div className="p-3 text-xs text-text-dim">{text}</div>;
}
