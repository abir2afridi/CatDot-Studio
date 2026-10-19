// Guided Mode (Part X1/X6, Phase 1 shell): a linear task list whose each
// step reflects the real workspace state. Steps that need not-yet-real
// features (decode, device install) show an honest "Planned · Phase N"
// badge and are not clickable — never fake progression.
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkBuildSign, deviceInstallSigned } from "../lib/apk";
import { describeError } from "../lib/opener";
import { useProjectStage, type Stage } from "../lib/project-stage";
import { useImportApk } from "../hooks/useImportApk";
import { useUi } from "../state/ui";
import { PlannedBadge } from "../components/PlannedBadge";

type StepStatus = "done" | "current" | "todo" | "planned";

interface StepDef {
  id: string;
  titleKey: string;
  descKey: string;
  /** stage at which this step is "done" (index in STEP_ORDER) */
  doneAt: number;
  /** optional real action button */
  action?: "openApk" | "buildSign" | "openExplorer" | "install";
  /** honest phase badge when the underlying feature is not real yet */
  plannedPhase?: number;
}

const STEP_ORDER: Stage[] = ["none", "imported", "edited", "built", "signed"];

const STEPS: StepDef[] = [
  { id: "import", titleKey: "guided.s1", descKey: "guided.s1desc", doneAt: 0, action: "openApk" },
  { id: "inspect", titleKey: "guided.s2", descKey: "guided.s2desc", doneAt: 1, action: "openExplorer" },
  { id: "edit", titleKey: "guided.s3", descKey: "guided.s3desc", doneAt: 2 },
  { id: "manifest", titleKey: "guided.s4", descKey: "guided.s4desc", doneAt: 2, plannedPhase: 3 },
  { id: "build", titleKey: "guided.s5", descKey: "guided.s5desc", doneAt: 3, action: "buildSign" },
  { id: "install", titleKey: "guided.s6", descKey: "guided.s6desc", doneAt: 4, action: "install" },
];

export function GuidedPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setLastBuild = useUi((s) => s.setLastBuild);
  const setNotice = useUi((s) => s.setNotice);
  const setPage = useUi((s) => s.setPage);
  const deviceSerial = useUi((s) => s.deviceSerial);
  const { stage, isLoading, pendingEdits } = useProjectStage();
  const { pickAndImport, isPending: importing } = useImportApk();

  const buildMut = useMutation({
    mutationFn: () => apkBuildSign(),
    onSuccess: (res) => {
      setLastBuild(res);
      const ok = res.steps.length > 0 && res.steps.every((s) => s.exit_code === 0);
      setNotice({ text: ok ? t("explorer.built") : t("explorer.buildFailed"), ok });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => {
      setLastBuild(null);
      setNotice({ text: describeError(e), ok: false });
    },
  });

  const installMut = useMutation({
    mutationFn: () => deviceInstallSigned(deviceSerial),
    onSuccess: (serial) => {
      setNotice({ text: t("device.installed", { serial }), ok: true });
    },
    onError: (e) => setNotice({ text: describeError(e), ok: false }),
  });

  const idx = STEP_ORDER.indexOf(stage);

  // doneAt semantics: a step is "done" past its stage, "current" at it.
  // Planned steps always show the honest badge — they never complete.
  const statusOf = (s: StepDef): StepStatus => {
    if (s.plannedPhase != null) return "planned";
    if (idx > s.doneAt) return "done";
    if (idx === s.doneAt) return "current";
    return "todo";
  };

  const statusChip = (st: StepStatus) => {
    switch (st) {
      case "done":
        return <span className="text-success">✓ {t("guided.done")}</span>;
      case "current":
        return <span className="text-accent">{t("guided.current")}</span>;
      default:
        return <span className="text-text-dim/60">{t("guided.todo")}</span>;
    }
  };

  return (
    <div className="h-full overflow-y-auto bg-bg p-6">
      <div className="mx-auto max-w-2xl">
        <h1 className="mb-1 text-xl font-semibold">{t("guided.title")}</h1>
        <p className="mb-4 text-sm text-text-dim">{t("guided.subtitle")}</p>
        {pendingEdits > 0 && (
          <p className="mb-3 text-sm text-accent">
            {t("status.pendingEdits", { count: pendingEdits })}
          </p>
        )}
        <ol className="space-y-3">
          {STEPS.map((s) => {
            const st = isLoading ? "todo" : statusOf(s);
            return (
              <li
                key={s.id}
                className={`rounded-lg border p-3 text-sm ${
                  st === "current"
                    ? "border-accent/40 bg-accent/5"
                    : "border-border bg-surface"
                }`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="font-medium">{t(s.titleKey)}</p>
                    <p className="mt-0.5 text-[11px] text-text-dim">{t(s.descKey)}</p>
                  </div>
                  <div className="shrink-0 text-[11px]">
                    {s.plannedPhase != null ? <PlannedBadge phase={s.plannedPhase} /> : statusChip(st)}
                  </div>
                </div>
                {st === "current" && s.action != null && (
                  <div className="mt-2">
                    {s.action === "openApk" && (
                      <button
                        type="button"
                        disabled={importing}
                        onClick={pickAndImport}
                        className="rounded-md bg-accent/20 px-3 py-1.5 text-xs text-accent hover:bg-accent/30 disabled:opacity-50"
                      >
                        {importing ? t("explorer.importing") : t("explorer.openApk")}
                      </button>
                    )}
                    {s.action === "openExplorer" && (
                      <button
                        type="button"
                        onClick={() => setPage("explorer")}
                        className="rounded-md bg-accent/20 px-3 py-1.5 text-xs text-accent hover:bg-accent/30"
                      >
                        {t("guided.openExplorer")}
                      </button>
                    )}
                    {s.action === "buildSign" && (
                      <button
                        type="button"
                        disabled={buildMut.isPending}
                        onClick={() => buildMut.mutate()}
                        className="rounded-md bg-accent/20 px-3 py-1.5 text-xs text-accent hover:bg-accent/30 disabled:opacity-50"
                      >
                        {buildMut.isPending ? t("explorer.building") : t("explorer.buildSign")}
                      </button>
                    )}
                    {s.action === "install" && (
                      <button
                        type="button"
                        disabled={installMut.isPending}
                        onClick={() => installMut.mutate()}
                        className="rounded-md bg-success/15 px-3 py-1.5 text-xs text-success hover:bg-success/25 disabled:opacity-50"
                      >
                        {installMut.isPending ? t("device.installing") : t("map.installBtn")}
                      </button>
                    )}
                  </div>
                )}
              </li>
            );
          })}
        </ol>
      </div>
    </div>
  );
}
