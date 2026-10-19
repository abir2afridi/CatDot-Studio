// Shared "Open APK…" action used by Explorer, Welcome cards, Project Map
// and Guided Mode — ONE import flow (picker + real apk_import), no variants.
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkImport } from "../lib/apk";
import { pickApkPath } from "../lib/pick";
import { describeError } from "../lib/opener";
import { useUi } from "../state/ui";

export function useImportApk() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setNotice = useUi((s) => s.setNotice);
  const setPage = useUi((s) => s.setPage);

  const mut = useMutation({
    mutationFn: (sourcePath: string) => apkImport(sourcePath),
    onSuccess: (rep) => {
      setNotice({ text: t("explorer.imported", { count: rep.entry_count }), ok: true });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
      setPage("explorer");
    },
    onError: (e) => setNotice({ text: describeError(e), ok: false }),
  });

  const pickAndImport = () => {
    void pickApkPath(t("explorer.openApk")).then((p) => {
      if (p != null) mut.mutate(p);
    });
  };

  return { pickAndImport, isPending: mut.isPending };
}
