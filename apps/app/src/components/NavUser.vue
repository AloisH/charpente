<script setup lang="ts">
// Sidebar footer: avatar + user identity opening a dropdown with the account
// actions (theme, language, logout). The avatar is generated locally from the
// email — initials on a deterministic color — because the CSP allows no
// third-party avatar services.
import { ChevronsUpDown, Languages, LogOut, Moon, Sun } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { useAuth } from "@/composables/useAuth";
import { useUiStore, type Locale, type Theme } from "@/stores/ui";

const { t } = useI18n();
const { user, logout } = useAuth();
const ui = useUiStore();
const { isMobile } = useSidebar();

const initials = computed(() => {
  const name = user.value?.display_name.trim() ?? "";
  const parts = name.split(/\s+/).filter(Boolean);
  const first = parts[0]?.[0] ?? user.value?.email[0] ?? "?";
  const second = parts.length > 1 ? (parts.at(-1)?.[0] ?? "") : "";
  return (first + second).toUpperCase();
});

// Deterministic hue from the email: same account, same color, everywhere.
const avatarColor = computed(() => {
  const email = user.value?.email ?? "";
  let hash = 0;
  for (const char of email) {
    hash = (hash * 31 + (char.codePointAt(0) ?? 0)) % 360;
  }
  return `oklch(0.55 0.15 ${hash})`;
});
</script>

<template>
  <SidebarMenu>
    <SidebarMenuItem>
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <SidebarMenuButton
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
          >
            <Avatar class="size-8 rounded-lg">
              <AvatarFallback
                class="rounded-lg font-medium text-white"
                :style="{ backgroundColor: avatarColor }"
              >
                {{ initials }}
              </AvatarFallback>
            </Avatar>
            <div class="grid flex-1 text-left text-sm leading-tight">
              <span class="truncate font-medium">{{ user?.display_name }}</span>
              <span class="truncate text-xs text-muted-foreground">{{ user?.email }}</span>
            </div>
            <ChevronsUpDown class="ml-auto size-4" />
          </SidebarMenuButton>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          class="w-(--reka-dropdown-menu-trigger-width) min-w-56 rounded-lg"
          :side="isMobile ? 'bottom' : 'right'"
          align="end"
          :side-offset="4"
        >
          <DropdownMenuLabel class="p-0 font-normal">
            <div class="flex items-center gap-2 px-2 py-1.5 text-left text-sm">
              <Avatar class="size-8 rounded-lg">
                <AvatarFallback
                  class="rounded-lg font-medium text-white"
                  :style="{ backgroundColor: avatarColor }"
                >
                  {{ initials }}
                </AvatarFallback>
              </Avatar>
              <div class="grid flex-1 leading-tight">
                <span class="truncate font-medium">{{ user?.display_name }}</span>
                <span class="truncate text-xs text-muted-foreground">{{ user?.email }}</span>
              </div>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <Sun v-if="ui.colorMode !== 'dark'" class="mr-2 size-4" />
              <Moon v-else class="mr-2 size-4" />
              {{ t("nav.theme") }}
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              <DropdownMenuRadioGroup
                :model-value="ui.theme"
                @update:model-value="ui.setTheme($event as Theme)"
              >
                <DropdownMenuRadioItem value="light">
                  {{ t("nav.themeLight") }}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="dark">
                  {{ t("nav.themeDark") }}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="auto">
                  {{ t("nav.themeSystem") }}
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <Languages class="mr-2 size-4" />
              {{ t("nav.language") }}
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent>
              <DropdownMenuRadioGroup
                :model-value="ui.locale"
                @update:model-value="ui.setLocale($event as Locale)"
              >
                <DropdownMenuRadioItem value="fr">Français</DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="en">English</DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="logout.mutate({})">
            <LogOut />
            {{ t("auth.logout") }}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </SidebarMenuItem>
  </SidebarMenu>
</template>
