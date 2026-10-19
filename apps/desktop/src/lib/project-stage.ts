// Shared real project-stage model (H2 Project Map / Guided Mode / cards).
// Derived ONLY from live backend state (`apk_status` + `apk_report`) — never
// from user clicks. One stage per workspace; the UI picks the next honest
// action from it.
import { useQuery } from "@tanstack/react-query";
import { apkReport, apkStatus } from "./apk";

export type Stage = "none" | "imported" | "edited" | "built" | "signed";

const stageOf = (
  imported: boolean,
  pendingEdits: number,
  hasExport: boolean,
  hasSigned: boolean,
): Stage => {
  if (!imported) return "none";
  if (hasSigned) return "signed";
  if (hasExport) return "built";
  if (pendingEdits > 0) return "edited";
  return "imported";
};

export interface ProjectStage {
  stage: Stage;
  pendingEdits: number;
  originalApk: string | null;
  exportedApk: string | null;
  signedApk: string | null;
  workspaceDir: string;
  /** report fields (null while the report query fails/loads) */
  report: { name: string; sha256: string; entryCount: number; totalSize: bigint } | null;
  isLoading: boolean;
}

export function useProjectStage(): ProjectStage {
  const status = useQuery({ queryKey: ["apk", "status"], queryFn: apkStatus, staleTime: 3_000 });
  const report = useQuery({ queryKey: ["apk", "report"], queryFn: apkReport, retry: false });

  const s = status.data;
  const r = report.data;
  return {
    stage: stageOf(s?.imported ?? false, s?.pending_edits ?? 0, s?.exported_apk != null, s?.signed_apk != null),
    pendingEdits: s?.pending_edits ?? 0,
    originalApk: s?.original_apk ?? null,
    exportedApk: s?.exported_apk ?? null,
    signedApk: s?.signed_apk ?? null,
    workspaceDir: s?.workspace_dir ?? "",
    report:
      r != null
        ? { name: r.name, sha256: r.sha256, entryCount: r.entry_count, totalSize: r.total_size }
        : null,
    isLoading: status.isLoading,
  };
}
