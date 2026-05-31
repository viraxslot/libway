import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { en } from "@/i18n/locales/en";
import { ru } from "@/i18n/locales/ru";

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    ru: { translation: ru },
  },
  lng: "en",
  fallbackLng: "en",
  returnNull: false,
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
