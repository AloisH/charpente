import { createI18n } from "vue-i18n";

import en from "@/i18n/locales/en.json";
import fr from "@/i18n/locales/fr.json";

const STORAGE_KEY = "charpente.locale";

function initialLocale(): "fr" | "en" {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "fr" || stored === "en") return stored;
  return navigator.language.startsWith("fr") ? "fr" : "en";
}

const shortDate = {
  short: {
    year: "numeric",
    month: "short",
    day: "numeric",
  },
} as const;

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale(),
  fallbackLocale: "en",
  messages: { fr, en },
  datetimeFormats: { fr: shortDate, en: shortDate },
  numberFormats: {
    fr: { decimal: { style: "decimal" } },
    en: { decimal: { style: "decimal" } },
  },
});

export function setLocale(locale: "fr" | "en"): void {
  i18n.global.locale.value = locale;
  localStorage.setItem(STORAGE_KEY, locale);
  document.documentElement.lang = locale;
}
