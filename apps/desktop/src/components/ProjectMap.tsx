// Project Map strip (Part H2): five honest steps derived from real workspace
// state; the single most useful next action is one click away. Install runs a
// real `adb install -r` on the first ready device (honest errors otherwise).
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkBuildSign, deviceInstallSigned } from "../lib/apk";
import { describeError } from "../lib/opener";
import { useProjectStage, type Stage } from "../lib/project-stage";
import { useImportApk } from "../hooks/useImportApk";
import { useUi } from "../state/ui";

const STEP_ORDER: Stage[] = ["none", "imported", "edited", "built", "signed"];

const stepLabelKey = (s: Stage): string => `map.${s}`;

/** Steps that count as "reached" for a given stage. */
const reached = (stage: Stage): number => STEP_ORDER.indexOf(stage);

export function ProjectMap() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setLastBuild = useUi((s) => s.setLastBuild);
  const setNotice = useUi((s) => s.setNotice);
  const deviceSerial = useUi((s) => s.deviceSerial);
  const { stage, pendingEdits, isLoading } = useProjectStage();
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
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setNotice({ text: describeError(e), ok: false }),
  });

  if (isLoading) return null;

  const idx = reached(stage);
  const steps = STEP_ORDER.filter((s) => s !== "none");

  return (
    <section
      className="rounded-lg border border-border bg-surface p-3"
      data-testid="project-map"
      aria-label={t("map.title")}
    >
      <div className="mb-2 flex items-baseline gap-2">
        <h2 className="text-[11px] font-semibold uppercase tracking-wide text-text-dim">
          {t("map.title")}
        </h2>
        {pendingEdits > 0 && (
          <span className="text-[11px] text-accent">
            {t("status.pendingEdits", { count: pendingEdits })}
          </span>
        )}
      </div>
      <ol className="flex flex-wrap items-center gap-1 text-[11px]">
        {steps.map((s, i) => {
          const done = i < idx;
          const current = i === idx;
          return (
            <li key={s} className="flex items-center gap-1">
              <span
                className={`rounded px-1.5 py-0.5 ${
                  done
                    ? "bg-success/15 text-success"
                    : current
                      ? "bg-accent/15 text-accent"
                      : "text-text-dim/70"
                }`}
              >
                {done ? "✓ " : current ? "▸ " : ""}
                {t(stepLabelKey(s))}
              </span>
              {i < steps.length - 1 && <span className="text-text-dim/40">›</span>}
            </li>
          );
        })}
      </ol>
      <NextStepCoach
        stage={stage}
        importing={importing}
        onOpenApk={pickAndImport}
        onBuild={() => buildMut.mutate()}
        building={buildMut.isPending}
        onInstall={() => installMut.mutate()}
        installing={installMut.isPending}
      />
    </section>
  );
}

function NextStepCoach({
  stage,
  importing,
  onOpenApk,
  onBuild,
  building,
  onInstall,
  installing,
}: {
  stage: Stage;
  importing: boolean;
  onOpenApk: () => void;
  onBuild: () => void;
  building: boolean;
  onInstall: () => void;
  installing: boolean;
}) {
  const { t } = useTranslation();

  let text: string;
  let action: React.ReactNode = null;
  switch (stage) {
    case "none":
      text = t("map.nextImport");
      action = (
        <button
          type="button"
          disabled={importing}
          onClick={onOpenApk}
          className="rounded-md bg-accent/20 px-2 py-0.5 text-accent hover:bg-accent/30 disabled:opacity-50"
        >
          {t("explorer.openApk")}
        </button>
      );
      break;
    case "imported":
      text = t("map.nextEdit");
      break;
    case "edited":
    case "built":
      text = t("map.nextBuild");
      action = (
        <button
          type="button"
          disabled={building}
          onClick={onBuild}
          className="rounded-md bg-accent/20 px-2 py-0.5 text-accent hover:bg-accent/30 disabled:opacity-50"
        >
          {building ? t("explorer.building") : t("explorer.buildSign")}
        </button>
      );
      break;
    case "signed":
      text = t("map.nextInstall");
      action = (
        <button
          type="button"
          disabled={installing}
          onClick={onInstall}
          className="rounded-md bg-success/15 px-2 py-0.5 text-success hover:bg-success/25 disabled:opacity-50"
        >
          {installing ? t("device.installing") : t("map.installBtn")}
        </button>
      );
      break;
  }

  return (
    <p className="mt-2 flex items-center gap-2 text-[11px] text-text-dim">
      <span className="font-medium text-text-dim/80">{t("map.next")}</span>
      {text}
      {action}
    </p>
  );
}
