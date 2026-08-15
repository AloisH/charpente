<script setup lang="ts">
// Layout for authenticated pages: shadcn-vue sidebar (collapsible to icons,
// sheet on mobile, state persisted in a cookie) + a slim header with the
// trigger. Nav items are gated by permissions via can().
import {
  Database,
  ExternalLink,
  Files,
  HardDrive,
  Languages,
  LayoutDashboard,
  LogOut,
  Mail,
  Monitor,
  Moon,
  Search,
  Sun,
  Users,
} from "@lucide/vue";
import { computed, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import CommandPalette from "@/components/CommandPalette.vue";
import DevEnvironmentTile from "@/components/DevEnvironmentTile.vue";
import ImpersonationTile from "@/components/ImpersonationTile.vue";
import NavUser from "@/components/NavUser.vue";
import VerifyEmailTile from "@/components/VerifyEmailTile.vue";
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
import { Separator } from "@/components/ui/separator";
import { useAuth, type Permission } from "@/composables/useAuth";
import { formatShortcut, useShortcut } from "@/composables/useShortcut";
import { openPalette, registerCommands } from "@/lib/command-registry";
import { useUiStore } from "@/stores/ui";

const props = defineProps<{
  /** Page title, shown in the header bar and mirrored into document.title. */
  title?: string;
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

watchEffect(() => {
  document.title =
    props.title === undefined ? t("common.appName") : `${props.title} · ${t("common.appName")}`;
});
// Statically false in production builds — the banner never ships.
const isDev = import.meta.env.DEV;
const { can, logout } = useAuth();
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

// Local infra UIs from docker-compose.dev.yml — dev only, they don't exist in
// prod. Opened in a new tab: iframing them would fight CSP and their own
// frame protections.
const devTools = [
  { href: "http://localhost:8025", label: () => t("nav.devMailpit"), icon: Mail },
  {
    href: "http://localhost:9001/rustfs/console/",
    label: () => t("nav.devRustfs"),
    icon: HardDrive,
  },
  { href: "http://localhost:8081", label: () => t("nav.devAdminer"), icon: Database },
];

if (isDev) {
  registerCommands({
    id: "dev-tools",
    label: () => t("nav.devTools"),
    order: 90,
    actions: devTools.map((tool) => ({
      id: tool.href,
      label: tool.label,
      icon: tool.icon,
      perform: () => window.open(tool.href, "_blank", "noopener"),
    })),
  });
}
</script>

<template>
  <SidebarProvider>
    <Sidebar collapsible="icon">
      <SidebarHeader class="pt-3 pb-0">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton as-child size="lg">
              <RouterLink to="/dashboard">
                <Avatar class="size-8 shrink-0 rounded-lg">
                  <AvatarFallback
                    class="rounded-lg bg-sidebar-primary text-base leading-none font-bold text-sidebar-primary-foreground select-none [&>*]:translate-x-[0.5px]"
                  >
                    <span>C</span>
                  </AvatarFallback>
                </Avatar>
                <span class="truncate font-semibold">{{ t("common.appName") }}</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem class="mt-1.5">
            <SidebarMenuButton
              :tooltip="t('command.search')"
              class="border border-input bg-background text-muted-foreground hover:bg-accent group-data-[collapsible=icon]:p-[7px]!"
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

        <SidebarGroup v-if="isDev">
          <SidebarGroupLabel>{{ t("nav.devTools") }}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem v-for="tool in devTools" :key="tool.href">
                <SidebarMenuButton as-child :tooltip="tool.label()">
                  <a :href="tool.href" target="_blank" rel="noopener">
                    <component :is="tool.icon" />
                    <span class="flex-1">{{ tool.label() }}</span>
                    <ExternalLink class="!size-3 text-muted-foreground" />
                  </a>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter>
        <ImpersonationTile />
        <VerifyEmailTile />
        <DevEnvironmentTile v-if="isDev" />
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>

    <SidebarInset>
      <header class="flex h-14 items-center gap-3 border-b border-border px-4">
        <SidebarTrigger />
        <template v-if="title !== undefined">
          <Separator orientation="vertical" class="!h-4" />
          <h1 class="text-base font-semibold">{{ title }}</h1>
        </template>
      </header>
      <main class="flex-1 p-6">
        <slot />
      </main>
    </SidebarInset>
    <CommandPalette />
  </SidebarProvider>
</template>
