// Pinia is for genuinely-client state only (theme, locale, UI prefs).
// Server data lives in TanStack Query — never mirror it here.
import { useColorMode } from "@vueuse/core";
import { defineStore } from "pinia";
import { computed } from "vue";

import { i18n, setLocale as applyLocale } from "@/i18n";

export type Theme = "light" | "dark" | "auto";
export type Locale = "fr" | "en";

export const useUiStore = defineStore("ui", () => {
  const colorMode = useColorMode({
    selector: "html",
    attribute: "class",
    modes: { light: "", dark: "dark" },
    storageKey: "charpente.theme",
  });

  /** The stored preference — "auto" means follow the system. */
  const theme = computed<Theme>({
    get: () => colorMode.store.value,
    set: (value) => {
      colorMode.value = value;
    },
  });

  const locale = computed(() => i18n.global.locale.value as Locale);

  function setTheme(value: Theme): void {
    theme.value = value;
  }

  function toggleTheme(): void {
    colorMode.value = colorMode.value === "dark" ? "light" : "dark";
  }

  function switchLocale(): void {
    setLocale(locale.value === "fr" ? "en" : "fr");
  }

  return { colorMode, theme, locale, setTheme, toggleTheme, setLocale, switchLocale };
});

function setLocale(value: Locale): void {
  applyLocale(value);
}
