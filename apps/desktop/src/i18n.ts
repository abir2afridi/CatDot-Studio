import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { en, bn } from "./i18n-resources";

// Phase 0 i18n skeleton: English complete, Bangla scaffold. Key sets must stay
// equal (enforced by a unit test); BN falls back to EN for any missing value
// via the `fallbackLng` below until its translation is finished in Phase 3.
const stored = localStorage.getItem("locale");
const initialLng = stored === "bn" ? "bn" : "en";

i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, bn: { translation: bn } },
  lng: initialLng,
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export default i18n;
