import path from "node:path";
import { fileURLToPath } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite";
import vue from "@vitejs/plugin-vue";
import VueRouter from "unplugin-vue-router/vite";
import { defineConfig } from "vite";

const dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [
    // Must come before the vue plugin: file-based, typed routes from src/pages.
    VueRouter({ routesFolder: "src/pages", dts: "src/typed-router.d.ts" }),
    vue(),
    tailwindcss(),
    VueI18nPlugin({
      include: [path.resolve(dirname, "src/i18n/locales/**")],
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(dirname, "src"),
    },
  },
  server: {
    port: 5173,
    // Same-origin API in dev too: cookies flow without CORS involvement.
    proxy: {
      "/api": "http://localhost:8080",
    },
  },
  build: {
    sourcemap: true,
  },
})
