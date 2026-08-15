<script setup lang="ts">
// Layout for authenticated pages: shadcn-vue sidebar (collapsible to icons,
// sheet on mobile, state persisted in a cookie) + a slim header with the
// trigger. Nav items are gated by permissions via can().
import {
  Files,
  FlaskConical,
  Languages,
  LayoutDashboard,
  LogOut,
  Monitor,
  Moon,
  Search,
  Sun,
  Users,
} from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import CommandPalette from "@/components/CommandPalette.vue";
import NavUser from "@/components/NavUser.vue";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { useAuth, type Permission } from "@/composables/useAuth";
import { formatShortcut, useShortcut } from "@/composables/useShortcut";
import { openPalette, registerCommands } from "@/lib/command-registry";
import { useUiStore } from "@/stores/ui";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
// Statically false in production builds — the banner never ships.
const isDev = import.meta.env.DEV;
const { user, can, logout } = useAuth();
const ui = useUiStore();

// App-wide shortcuts (hints rendered via formatShortcut).
useShortcut("Mod+Shift+L", () => ui.toggleTheme());
useShortcut("Mod+Shift+E", () => ui.switchLocale());

// Global palette commands. Pages add their own groups the same way.
registerCommands({
  id: "navigation",
  label: () => t("command.navigation"),
  order: 0,
  actions: [
    {
      id: "dashboard",
      label: () => t("nav.dashboard"),
      icon: LayoutDashboard,
      perform: () => router.push("/dashboard"),
    },
    {
      id: "uploads",
      label: () => t("nav.uploads"),
      icon: Files,
      perform: () => router.push("/uploads"),
    },
    {
      id: "admin",
      label: () => t("nav.admin"),
      icon: Users,
      when: () => can("manage-users"),
      perform: () => router.push("/admin/users"),
    },
  ],
});

registerCommands({
  id: "preferences",
  label: () => t("command.preferences"),
  order: 10,
  actions: [
    {
      id: "theme-light",
      label: () => `${t("nav.theme")} : ${t("nav.themeLight")}`,
      icon: Sun,
      perform: () => ui.setTheme("light"),
    },
    {
      id: "theme-dark",
      label: () => `${t("nav.theme")} : ${t("nav.themeDark")}`,
      icon: Moon,
      shortcut: formatShortcut("Mod+Shift+L"),
      perform: () => ui.setTheme("dark"),
    },
    {
      id: "theme-system",
      label: () => `${t("nav.theme")} : ${t("nav.themeSystem")}`,
      icon: Monitor,
      perform: () => ui.setTheme("auto"),
    },
    {
      id: "language",
      label: () => (ui.locale === "fr" ? "English" : "Français"),
      icon: Languages,
      shortcut: formatShortcut("Mod+Shift+E"),
      perform: () => ui.switchLocale(),
    },
  ],
});

registerCommands({
  id: "session",
  label: () => t("command.session"),
  order: 20,
  actions: [
    {
      id: "logout",
      label: () => t("auth.logout"),
      icon: LogOut,
      perform: () => logout.mutate({}),
    },
  ],
});

interface NavItem {
  to: string;
  label: string;
  icon: typeof LayoutDashboard;
  permission?: Permission;
}

const navItems = computed<NavItem[]>(() =>
  [
    { to: "/dashboard", label: t("nav.dashboard"), icon: LayoutDashboard },
    { to: "/uploads", label: t("nav.uploads"), icon: Files },
    {
      to: "/admin/users",
      label: t("nav.admin"),
      icon: Users,
      permission: "manage-users" as const,
    },
  ].filter((item) => item.permission === undefined || can(item.permission)),
);

const isActive = (to: string): boolean => route.path.startsWith(to);
</script>

<template>
  <SidebarProvider>
    <Sidebar collapsible="icon">
      <SidebarHeader class="gap-2 pt-3">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton as-child size="lg">
              <RouterLink to="/dashboard">
                <Avatar class="size-8 shrink-0 rounded-lg">
                  <AvatarFallback
                    class="rounded-lg bg-sidebar-primary font-semibold text-sidebar-primary-foreground"
                  >
                    c
                  </AvatarFallback>
                </Avatar>
                <span class="truncate font-semibold">{{ t("common.appName") }}</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              :tooltip="t('command.search')"
              class="border border-input bg-background text-muted-foreground hover:bg-accent"
              @click="openPalette()"
            >
              <Search />
              <span class="flex-1 text-left">{{ t("command.search") }}</span>
              <kbd
                class="pointer-events-none rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium group-data-[collapsible=icon]:hidden"
              >
                {{ formatShortcut("Mod+K") }}
              </kbd>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>{{ t("nav.navigation") }}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem v-for="item in navItems" :key="item.to">
                <SidebarMenuButton as-child :is-active="isActive(item.to)" :tooltip="item.label">
                  <RouterLink :to="item.to">
                    <component :is="item.icon" />
                    <span>{{ item.label }}</span>
                  </RouterLink>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter>
        <div
          v-if="isDev"
          class="relative flex items-center gap-2.5 overflow-hidden rounded-lg border border-amber-500/30 bg-gradient-to-r from-amber-500/15 via-orange-500/10 to-amber-500/15 px-2.5 py-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0"
          :title="t('common.devBanner')"
        >
          <span
            class="pointer-events-none absolute inset-y-0 w-1/3 animate-dev-shimmer bg-gradient-to-r from-transparent via-amber-300/25 to-transparent group-data-[collapsible=icon]:hidden"
          />
          <span
            class="relative flex size-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-amber-500 to-orange-600 text-white shadow-sm"
          >
            <FlaskConical class="size-4" />
            <span class="absolute -top-0.5 -right-0.5 flex size-2">
              <span
                class="absolute inline-flex size-full animate-ping rounded-full bg-amber-400 opacity-75"
              />
              <span class="relative inline-flex size-2 rounded-full bg-amber-500" />
            </span>
          </span>
          <div class="relative grid leading-tight group-data-[collapsible=icon]:hidden">
            <span
              class="bg-gradient-to-r from-amber-600 to-orange-600 bg-clip-text text-xs font-semibold text-transparent dark:from-amber-400 dark:to-orange-400"
            >
              {{ t("common.devBanner") }}
            </span>
            <span class="font-mono text-[10px] text-muted-foreground">localhost</span>
          </div>
        </div>
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>

    <SidebarInset>
      <header class="flex h-14 items-center gap-3 border-b border-border px-4">
        <SidebarTrigger />
        <div class="ml-auto text-sm text-muted-foreground">
          {{ user?.display_name }}
        </div>
      </header>
      <main class="flex-1 p-6">
        <slot />
      </main>
    </SidebarInset>
    <CommandPalette />
  </SidebarProvider>
</template>
