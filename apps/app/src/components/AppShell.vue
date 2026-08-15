<script setup lang="ts">
// Layout for authenticated pages: top nav, theme/locale toggles, logout.
import { Languages, LogOut, Moon, Sun } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import Can from "@/components/Can.vue";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/composables/useAuth";
import { useShortcut } from "@/composables/useShortcut";
import { useUiStore } from "@/stores/ui";

const { t } = useI18n();
const { user, logout } = useAuth();
const ui = useUiStore();

// Example shortcut: Mod+Shift+L toggles the theme.
useShortcut("Mod+Shift+L", () => ui.toggleTheme());
</script>

<template>
  <div class="min-h-screen">
    <header class="border-b border-border">
      <nav class="mx-auto flex max-w-6xl items-center justify-between px-6 py-3">
        <div class="flex items-center gap-6">
          <RouterLink to="/dashboard" class="font-semibold">
            {{ t("common.appName") }}
          </RouterLink>
          <RouterLink
            to="/dashboard"
            class="text-sm text-muted-foreground hover:text-foreground"
            active-class="!text-foreground"
          >
            {{ t("nav.dashboard") }}
          </RouterLink>
          <RouterLink
            to="/uploads"
            class="text-sm text-muted-foreground hover:text-foreground"
            active-class="!text-foreground"
          >
            {{ t("nav.uploads") }}
          </RouterLink>
          <Can permission="manage-users">
            <RouterLink
              to="/admin/users"
              class="text-sm text-muted-foreground hover:text-foreground"
              active-class="!text-foreground"
            >
              {{ t("nav.admin") }}
            </RouterLink>
          </Can>
        </div>

        <div class="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            :aria-label="t('nav.toggleTheme')"
            @click="ui.toggleTheme()"
          >
            <Sun v-if="ui.colorMode === 'dark'" class="size-4" />
            <Moon v-else class="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            :aria-label="t('nav.language')"
            @click="ui.switchLocale()"
          >
            <Languages class="size-4" />
          </Button>
          <span class="px-2 text-sm text-muted-foreground">{{ user?.display_name }}</span>
          <Button
            variant="ghost"
            size="icon"
            :aria-label="t('auth.logout')"
            @click="logout.mutate({})"
          >
            <LogOut class="size-4" />
          </Button>
        </div>
      </nav>
    </header>

    <main class="mx-auto max-w-6xl px-6 py-8">
      <slot />
    </main>
  </div>
</template>
