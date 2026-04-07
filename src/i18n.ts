import i18next from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import en from "./locales/en.json";
import uk from "./locales/uk.json";

i18next
  .use(LanguageDetector)
  .init({
    fallbackLng: "uk",
    debug: false,
    resources: {
      en: { translation: en },
      uk: { translation: uk },
    },
    interpolation: {
      escapeValue: false,
    },
  });

export default i18next;
export const t = i18next.t.bind(i18next);
