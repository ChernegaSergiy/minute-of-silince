import i18next from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import en from "../locales/en.json";
import uk from "../locales/uk.json";

const detector = new LanguageDetector();
detector.addDetector({
  name: "navigator",
  lookup: () => navigator.language.split("-")[0],
});

i18next
  .use(detector)
  .init({
    fallbackLng: "uk",
    detection: {
      order: ["navigator"],
      caches: [],
    },
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
