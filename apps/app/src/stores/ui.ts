// Pinia is for genuinely-client state only (theme, locale, UI prefs).
// Server data lives in TanStack Query — never mirror it here.
import { useColorMode } from "@vueuse/core";
import { defineStore } from "pinia";
import { computed } from "vue";

import { i18n, setLocale } from "@/i18n";

export const useUiStore = defineStore("ui", () => {
  const colorMode = useColorMode({
    selector: "html",
    attribute: "class",
    modes: { light: "", dark: "dark" },
    storageKey: "charpente.theme",
  });

  const locale = computed(() => i18n.global.locale.value);

  function toggleTheme(): void {
    colorMode.value = colorMode.value === "dark" ? "light" : "dark";
  }

  function switchLocale(): void {
    setLocale(locale.value === "fr" ? "en" : "fr");
  }

  return { colorMode, locale, toggleTheme, switchLocale };
});
