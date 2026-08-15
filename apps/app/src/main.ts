import { VueQueryPlugin } from "@tanstack/vue-query";
import { createPinia } from "pinia";
import { createApp } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import { routes } from "vue-router/auto-routes";

import App from "@/App.vue";
import { i18n } from "@/i18n";
import { installAuthGuard } from "@/lib/auth-guard";
import { queryClient } from "@/lib/query";

import "@/assets/main.css";

const router = createRouter({
  history: createWebHistory(),
  routes,
});

installAuthGuard(router);

createApp(App)
  .use(createPinia())
  .use(router)
  .use(i18n)
  .use(VueQueryPlugin, { queryClient })
  .mount("#app");
