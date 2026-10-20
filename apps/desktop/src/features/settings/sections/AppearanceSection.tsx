// Appearance: theme + locale, persisted immediately as global settings.
import { useTranslation } from "react-i18next";
import { settingsSet } from "../../../lib/ipc";
import { useUi } from "../../../state/ui";
import { Field, Toggle } from "../SettingsControls";

export function AppearanceSection() {
  const { t, i18n } = useTranslation();
  const theme = useUi((s) => s.theme);
  const setTheme = useUi((s) => s.setTheme);
  const locale = useUi((s) => s.locale);
  const setLocale = useUi((s) => s.setLocale);

  function applyTheme(next: "dark" | "light") {
    setTheme(next);
    document.documentElement.dataset.theme = next;
    void settingsSet("ui.theme", next, "global");
  }

  function applyLocale(next: "en" | "bn") {
    setLocale(next);
    void i18n.changeLanguage(next);
    void settingsSet("ui.locale", next, "global");
  }

  return (
    <section className="mt-6 space-y-4">
      <Field label={t("settings.theme")}>
        <div className="flex gap-2">
          <Toggle active={theme === "dark"} onClick={() => applyTheme("dark")}>
            {t("settings.themeDark")}
          </Toggle>
          <Toggle active={theme === "light"} onClick={() => applyTheme("light")}>
            {t("settings.themeLight")}
          </Toggle>
        </div>
      </Field>

      <Field label={t("settings.locale")}>
        <div className="flex gap-2">
          <Toggle active={locale === "en"} onClick={() => applyLocale("en")}>
            English
          </Toggle>
          <Toggle active={locale === "bn"} onClick={() => applyLocale("bn")}>
            বাংলা
          </Toggle>
        </div>
      </Field>
    </section>
  );
}
