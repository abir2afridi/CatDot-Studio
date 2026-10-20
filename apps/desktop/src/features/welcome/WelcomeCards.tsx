// H2 dashboard cards: Original APK (protected, read-only) + build outputs.
// Every field comes from real workspace state; "Reveal in folder" opens the
// actual file in Windows Explorer via the opener plugin.
import { useTranslation } from "react-i18next";
import { useProjectStage } from "../../lib/project-stage";
import { revealPath, describeError } from "../../lib/opener";
import { useImportApk } from "../../hooks/useImportApk";
import { useUi } from "../../state/ui";

export function WelcomeCards() {
  const { t } = useTranslation();
  const { stage, report, originalApk, exportedApk, signedApk, pendingEdits } =
    useProjectStage();
  const { pickAndImport, isPending } = useImportApk();
  const setNotice = useUi((s) => s.setNotice);

  const reveal = (path: string) => {
    void revealPath(path).catch((e) => setNotice({ text: describeError(e), ok: false }));
  };

  if (stage === "none") {
    return (
      <section className="rounded-lg border border-dashed border-border p-6 text-center text-sm text-text-dim">
        <p className="mb-3">{t("card.empty")}</p>
        <button
          type="button"
          disabled={isPending}
          onClick={pickAndImport}
          className="rounded-md bg-accent/20 px-4 py-2 font-medium text-accent hover:bg-accent/30 disabled:opacity-50"
        >
          {isPending ? t("explorer.importing") : t("explorer.openApk")}
        </button>
      </section>
    );
  }

  return (
    <div className="grid gap-3 lg:grid-cols-2">
      <section className="rounded-lg border border-border bg-surface p-3 text-sm">
        <header className="mb-2 flex items-center gap-2">
          <h2 className="font-semibold">{t("card.titleOriginal")}</h2>
          <span className="rounded bg-surface-2 px-1.5 py-0.5 text-[11px] text-text-dim">
            {t("card.readOnly")}
          </span>
        </header>
        {report != null ? (
          <dl className="space-y-1 text-[11px]">
            <Row label={t("card.name")} value={report.name} />
            <Row label={t("card.sha")} value={report.sha256} mono title={report.sha256} />
            <Row
              label={t("card.size")}
              value={`${formatBytes(report.totalSize)} · ${t("card.entries", { count: report.entryCount })}`}
            />
            <Row
              label={t("card.pendingLabel")}
              value={t("status.pendingEdits", { count: pendingEdits })}
            />
          </dl>
        ) : (
          <p className="text-[11px] text-text-dim">{t("common.loading")}</p>
        )}
        {originalApk != null && (
          <button
            type="button"
            onClick={() => reveal(originalApk)}
            className="mt-2 rounded-md bg-surface-2 px-2 py-1 text-[11px] hover:bg-surface-2/70"
          >
            {t("card.reveal")}
          </button>
        )}
      </section>

      <section className="rounded-lg border border-border bg-surface p-3 text-sm">
        <header className="mb-2 flex items-center gap-2">
          <h2 className="font-semibold">{t("card.titleOutputs")}</h2>
          <span className="text-[11px] text-text-dim">{t("card.outputsHint")}</span>
        </header>
        {signedApk == null && exportedApk == null ? (
          <p className="text-[11px] text-text-dim">{t("card.noOutputs")}</p>
        ) : (
          <dl className="space-y-1 text-[11px]">
            {exportedApk != null && (
              <Row
                label={t("card.exported")}
                value={exportedApk}
                mono
                title={exportedApk}
              />
            )}
            {signedApk != null && (
              <Row label={t("card.signed")} value={signedApk} mono title={signedApk} />
            )}
          </dl>
        )}
        <div className="mt-2 flex gap-2">
          {exportedApk != null && (
            <button
              type="button"
              onClick={() => reveal(exportedApk)}
              className="rounded-md bg-surface-2 px-2 py-1 text-[11px] hover:bg-surface-2/70"
            >
              {t("card.reveal")}
            </button>
          )}
          {signedApk != null && (
            <button
              type="button"
              onClick={() => reveal(signedApk)}
              className="rounded-md bg-success/15 px-2 py-1 text-[11px] text-success hover:bg-success/25"
            >
              {t("card.revealSigned")}
            </button>
          )}
        </div>
      </section>
    </div>
  );
}

function Row({
  label,
  value,
  mono,
  title,
}: {
  label: string;
  value: string;
  mono?: boolean;
  title?: string;
}) {
  return (
    <div className="flex gap-2">
      <dt className="w-24 shrink-0 text-text-dim">{label}</dt>
      <dd className={`min-w-0 flex-1 truncate ${mono ? "font-mono" : ""}`} title={title}>
        {value}
      </dd>
    </div>
  );
}

function formatBytes(n: bigint): string {
  const mb = Number(n) / (1024 * 1024);
  if (mb >= 1) return `${mb.toFixed(2)} MiB`;
  return `${(Number(n) / 1024).toFixed(1)} KiB`;
}
