import tailwindcss from "@tailwindcss/vite";

// The public-facing vitrine: SSR, FR/EN with localized prefixes, SEO defaults.
export default defineNuxtConfig({
  compatibilityDate: "2026-08-01",
  modules: ["@nuxtjs/i18n", "@nuxtjs/sitemap", "@nuxtjs/robots"],

  css: ["~/assets/css/main.css"],
  vite: {
    plugins: [tailwindcss()],
  },

  site: {
    // Overridden per deployment via NUXT_PUBLIC_SITE_URL.
    url: "https://example.com",
    name: "charpente",
  },

  runtimeConfig: {
    public: {
      // Where "Open the app" points: the Vite dev server locally, the app
      // domain in prod (override via NUXT_PUBLIC_APP_URL).
      appUrl: "http://localhost:5173",
    },
  },

  i18n: {
    baseUrl: "https://example.com",
    defaultLocale: "fr",
    strategy: "prefix_except_default",
    locales: [
      { code: "fr", language: "fr-FR", name: "Français", file: "fr.json" },
      { code: "en", language: "en-US", name: "English", file: "en.json" },
    ],
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: "site_locale",
      redirectOn: "root",
    },
  },

  app: {
    head: {
      titleTemplate: "%s · charpente",
      meta: [
        { name: "viewport", content: "width=device-width, initial-scale=1" },
        { property: "og:type", content: "website" },
      ],
      link: [{ rel: "icon", href: "/favicon.svg", type: "image/svg+xml" }],
    },
  },

  routeRules: {
    "/": { prerender: false },
  },
});
