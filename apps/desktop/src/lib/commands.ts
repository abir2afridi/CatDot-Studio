// Command registry for the palette (Part X1): commands come from real
// features only; disabled entries carry an honest reason (never hidden).
// Mutation commands surface their structured result in a status-bar notice.
import { useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apkExport, apkBuildSign, apkImport, apkRevertAll, apkStatus, checkpointCreate, deviceInstallSigned } from "./apk";
import { pickApkPath } from "./pick";
import { IpcError } from "./ipc";
import { revealPath } from "./opener";
import { useUi, type PaletteCommand } from "../state/ui";

const describe = (e: unknown): string =>
  e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);

const leafOf = (p: string): string => {
  const i = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return i < 0 ? p : p.slice(i + 1);
};

interface RawCmd {
  id: string;
  titleKey: string;
  run: () => void;
  disabledReason?: string;
  /** extra english keywords for matching (e.g. shortcut names). */
  hay?: string;
}

function listOf(
  raws: ReadonlyArray<RawCmd>,
  translate: (key: string) => string,
): PaletteCommand[] {
  return raws.map((c) => ({
    id: c.id,
    titleKey: c.titleKey,
    disabledReason: c.disabledReason,
    run: c.run,
    title: translate(c.titleKey),
    haystack: [c.id, translate(c.titleKey), c.hay ?? ""].join(" ").toLowerCase(),
  }));
}

/** Hook: builds the (stable) command list the palette renders. */
export function useCommands(): PaletteCommand[] {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setPage = useUi((s) => s.setPage);
  const openSearch = useUi((s) => s.openSearch);
  const toggleExplorer = useUi((s) => s.toggleExplorer);
  const toggleBottom = useUi((s) => s.toggleBottom);
  const activeFile = useUi((s) => s.activeFile);
  const draft = useUi((s) => (activeFile == null ? null : s.drafts[activeFile]));
  const requestSave = useUi((s) => s.requestSave);
  const closeFile = useUi((s) => s.closeFile);
  const setLastBuild = useUi((s) => s.setLastBuild);
  const setNotice = useUi((s) => s.setNotice);
  const deviceSerial = useUi((s) => s.deviceSerial);

  const invalidateApk = useCallback(
    () => void queryClient.invalidateQueries({ queryKey: ["apk"] }),
    [queryClient],
  );

  const importMut = useMutation({
    mutationFn: (sourcePath: string) => apkImport(sourcePath),
    onSuccess: (rep) => {
      setNotice({ text: t("explorer.imported", { count: rep.entry_count }), ok: true });
      invalidateApk();
      setPage("explorer");
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const exportMut = useMutation({
    mutationFn: () => apkExport(),
    onSuccess: (res) => {
      setNotice({ text: t("explorer.exported", { name: leafOf(res.path) }), ok: true });
      invalidateApk();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const buildMut = useMutation({
    mutationFn: () => apkBuildSign(),
    onSuccess: (res) => {
      setLastBuild(res);
      const ok = res.steps.length > 0 && res.steps.every((s) => s.exit_code === 0);
      setNotice({ text: ok ? t("explorer.built") : t("explorer.buildFailed"), ok });
      invalidateApk();
    },
    onError: (e) => {
      setLastBuild(null);
      setNotice({ text: describe(e), ok: false });
    },
  });

  const revertAllMut = useMutation({
    mutationFn: () => apkRevertAll(),
    onSuccess: () => {
      setNotice({ text: t("explorer.revertAllDone"), ok: true });
      invalidateApk();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  // Reveal-in-folder targets (real workspace files only).
  const status = useQuery({ queryKey: ["apk", "status"], queryFn: apkStatus, staleTime: 5_000 });

  const installMut = useMutation({
    mutationFn: () => deviceInstallSigned(deviceSerial),
    onSuccess: (serial) => {
      setNotice({ text: t("device.installed", { serial }), ok: true });
      invalidateApk();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const checkpointMut = useMutation({
    mutationFn: () => checkpointCreate(),
    onSuccess: (cp) => {
      setNotice({ text: t("hist.created", { count: cp.patch_count }), ok: true });
      invalidateApk();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });
  const revealOriginal = useCallback(() => {
    const p = status.data?.original_apk;
    if (p != null) {
      void revealPath(p).catch((e) => setNotice({ text: describe(e), ok: false }));
    }
  }, [status.data, setNotice]);
  const revealSigned = useCallback(() => {
    const p = status.data?.signed_apk;
    if (p != null) {
      void revealPath(p).catch((e) => setNotice({ text: describe(e), ok: false }));
    }
  }, [status.data, setNotice]);

  const pickAndImport = useCallback(() => {
    void pickApkPath(t("explorer.openApk")).then((p) => {
      if (p != null) importMut.mutate(p);
    });
    // dispatch-only callback: the mutation lives for the palette's lifetime
  }, [t, importMut]);

  const nothingKey = t("cmd.nothingToSave");

  return useMemo(
    () =>
      listOf(
        [
          { id: "view.guided", titleKey: "cmd.goGuided", run: () => setPage("guided") },
          { id: "view.explorer", titleKey: "cmd.goExplorer", run: () => setPage("explorer") },
          {
            id: "apk.search",
            titleKey: "cmd.searchApk",
            hay: "ctrl+shift+f find text",
            run: () => openSearch(),
          },
          { id: "view.devices", titleKey: "cmd.goDevices", run: () => setPage("devices") },
          { id: "view.system", titleKey: "cmd.goSystem", run: () => setPage("system") },
          { id: "view.settings", titleKey: "cmd.goSettings", run: () => setPage("settings") },
          { id: "view.toggleSidebar", titleKey: "cmd.toggleSidebar", hay: "ctrl+b", run: () => toggleExplorer() },
          { id: "view.togglePanel", titleKey: "cmd.togglePanel", hay: "ctrl+j", run: () => toggleBottom() },
          {
            id: "view.diff",
            titleKey: "cmd.goDiff",
            run: () => {
              const ui = useUi.getState();
              ui.setActiveBottomTab("diff");
              if (!ui.bottomOpen) toggleBottom();
            },
          },
          { id: "apk.import", titleKey: "cmd.importApk", run: () => pickAndImport() },
          { id: "apk.export", titleKey: "cmd.exportApk", run: () => exportMut.mutate() },
          { id: "apk.build", titleKey: "cmd.buildSign", run: () => buildMut.mutate() },
          { id: "apk.revertAll", titleKey: "cmd.revertAll", run: () => revertAllMut.mutate() },
          {
            id: "apk.revealOriginal",
            titleKey: "cmd.revealOriginal",
            disabledReason:
              status.data?.original_apk == null ? t("cmd.noOriginal") : undefined,
            run: () => revealOriginal(),
          },
          {
            id: "apk.revealSigned",
            titleKey: "cmd.revealSigned",
            disabledReason: status.data?.signed_apk == null ? t("cmd.noSigned") : undefined,
            run: () => revealSigned(),
          },
          {
            id: "apk.installDevice",
            titleKey: "cmd.installDevice",
            disabledReason: status.data?.signed_apk == null ? t("cmd.noSigned") : undefined,
            run: () => installMut.mutate(),
          },
          { id: "apk.saveCheckpoint", titleKey: "cmd.saveCheckpoint", run: () => checkpointMut.mutate() },
          {
            id: "editor.save",
            titleKey: "cmd.saveTab",
            disabledReason: activeFile == null || draft == null ? nothingKey : undefined,
            run: () => requestSave(),
          },
          {
            id: "editor.closeTab",
            titleKey: "cmd.closeTab",
            disabledReason: activeFile == null ? t("cmd.nothingToClose") : undefined,
            run: () => {
              if (activeFile != null) closeFile(activeFile);
            },
          },
        ],
        t,
      ),
    [
      t,
      setPage,
      openSearch,
      toggleExplorer,
      toggleBottom,
      pickAndImport,
      exportMut,
      buildMut,
      revertAllMut,
      activeFile,
      draft,
      requestSave,
      closeFile,
      nothingKey,
      revealOriginal,
      revealSigned,
      installMut,
      checkpointMut,
      status.data?.original_apk,
      status.data?.signed_apk,
    ],
  );
}
