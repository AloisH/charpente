<script setup lang="ts">
// The amber "you are on localhost" tile at the bottom of the sidebar.
// Extracted from AppShell so it can read the sidebar state for its
// collapsed-mode tooltip (useSidebar only works below SidebarProvider).
import { FlaskConical } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { useSidebar } from "@/components/ui/sidebar";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

const { t } = useI18n();
const { isMobile, state } = useSidebar();
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <div
        class="relative flex items-center gap-2.5 overflow-hidden rounded-lg border border-amber-500/30 bg-gradient-to-r from-amber-500/15 via-orange-500/10 to-amber-500/15 px-2.5 py-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0"
      >
        <span
          class="pointer-events-none absolute inset-y-0 w-1/3 animate-dev-shimmer bg-gradient-to-r from-transparent via-amber-300/25 to-transparent group-data-[collapsible=icon]:hidden"
        />
        <span
          class="relative flex size-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-amber-500 to-orange-600 text-white shadow-sm"
        >
          <FlaskConical class="size-4" />
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
    </TooltipTrigger>
    <TooltipContent side="right" align="center" :hidden="state !== 'collapsed' || isMobile">
      {{ t("common.devBanner") }} — localhost
    </TooltipContent>
  </Tooltip>
</template>
