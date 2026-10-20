// Settings shell: appearance, access level, AI provider, app footer.
import { useTranslation } from "react-i18next";
import { AccessSection } from "./sections/AccessSection";
import { AiProviderSection } from "./sections/AiProviderSection";
import { AppearanceSection } from "./sections/AppearanceSection";
import { AboutFooter } from "./sections/AboutFooter";

export function SettingsPage() {
  const { t } = useTranslation();
  return (
    <div className="max-w-2xl p-6" data-testid="settings-page">
      <h1 className="text-xl font-semibold">{t("settings.title")}</h1>
      <p className="mt-1 text-sm text-text-dim">{t("settings.subtitle")}</p>
      <AppearanceSection />
      <AccessSection />
      <AiProviderSection />
      <AboutFooter />
    </div>
  );
}
