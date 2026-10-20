// Welcome surface: project map + next-action cards when no file is open.
import { useTranslation } from "react-i18next";
import { ProjectMap } from "../welcome/ProjectMap";
import { WelcomeCards } from "../welcome/WelcomeCards";

export function Welcome() {
  const { t } = useTranslation();
  return (
    <div className="h-full overflow-y-auto bg-bg p-4">
      <div className="mx-auto max-w-3xl space-y-3">
        <p className="text-2xl font-semibold">{t("common.appName")}</p>
        <p className="text-sm text-text-dim">{t("editor.welcome")}</p>
        <ProjectMap />
        <WelcomeCards />
      </div>
    </div>
  );
}
