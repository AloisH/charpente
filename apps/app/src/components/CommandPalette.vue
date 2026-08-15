<script setup lang="ts">
// The Cmd+K palette. Renders whatever the registry holds (see
// lib/command-registry.ts) — it has no knowledge of the actions themselves.
import { useI18n } from "vue-i18n";

import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command";
import { useShortcut } from "@/composables/useShortcut";
import {
  closePalette,
  openPalette,
  paletteOpen,
  visibleGroups,
  type CommandAction,
} from "@/lib/command-registry";

const { t } = useI18n();

useShortcut("Mod+K", () => openPalette());

async function run(action: CommandAction): Promise<void> {
  closePalette();
  await Promise.resolve(action.perform());
}
</script>

<template>
  <CommandDialog
    :open="paletteOpen"
    :title="t('command.title')"
    :description="t('command.placeholder')"
    @update:open="paletteOpen = $event"
  >
    <CommandInput :placeholder="t('command.placeholder')" />
    <CommandList>
      <CommandEmpty>{{ t("command.empty") }}</CommandEmpty>
      <CommandGroup v-for="group in visibleGroups" :key="group.id" :heading="group.label()">
        <CommandItem
          v-for="action in group.actions"
          :key="action.id"
          :value="`${group.id}:${action.id} ${action.label()}`"
          @select="run(action)"
        >
          <component :is="action.icon" v-if="action.icon" class="mr-2 size-4" />
          <span>{{ action.label() }}</span>
          <CommandShortcut v-if="action.shortcut">{{ action.shortcut }}</CommandShortcut>
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </CommandDialog>
</template>
