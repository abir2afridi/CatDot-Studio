// Build tab feed: real per-step exit codes + alignment/signature checks.
import { useTranslation } from "react-i18next";
import { useUi } from "../../state/ui";
import { PanelEmpty } from "./PanelEmpty";

export function BuildFeed() {
  const { t } = useTranslation();
  const lastBuild = useUi((s) => s.lastBuild);
  if (lastBuild == null) return <PanelEmpty text={t("panel.noBuildYet")} />;
  return (
    <div className="h-full overflow-y-auto font-mono text-[11px] leading-4">
      {lastBuild.steps.map((s) => (
        <div key={s.step} className="px-2 py-0.5">
          <span className={s.exit_code === 0 ? "text-success" : "text-danger"}>
            [{s.exit_code}]
          </span>
          <span className="text-text-dim"> {s.step}</span>
          {s.detail !== "" && (
            <div className="whitespace-pre-wrap pl-8 text-text-dim">{s.detail}</div>
          )}
        </div>
      ))}
      {lastBuild.aligned_verified && (
        <div className="px-2 py-0.5 text-success">✓ {t("explorer.alignedVerified")}</div>
      )}
      {lastBuild.signature_verified && (
        <div className="px-2 py-0.5 text-success">✓ {t("explorer.signatureVerified")}</div>
      )}
    </div>
  );
}
