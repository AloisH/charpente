// The command palette's registry. The palette scales with the app because
// commands are *registered*, not hardcoded: AppShell registers the global ones
// (navigation, preferences, session) and any page can add its own —
// registrations made inside a component are dropped automatically when it
// unmounts.
//
//   registerCommands({
//     id: "uploads",
//     label: () => t("command.quickActions"),
//     actions: [{ id: "upload", label: () => t("..."), icon: Upload, perform: ... }],
//   });
import { computed, onScopeDispose, reactive, ref, type Component } from "vue";

export interface CommandAction {
  id: string;
  /** A function so labels re-render when the locale changes. */
  label: () => string;
  icon?: Component;
  /** Display-only hint, e.g. "⌘⇧L". */
  shortcut?: string;
  /** Hide the action conditionally (e.g. permission checks). */
  when?: () => boolean;
  /** Return value is ignored — router.push promises etc. are fine. */
  perform: () => unknown;
}

export interface CommandGroupDef {
  id: string;
  label: () => string;
  /** Lower comes first; global groups use 0–99, pages default to 100. */
  order?: number;
  actions: CommandAction[];
}

const groups = reactive(new Map<string, CommandGroupDef>());

export const paletteOpen = ref(false);

export function openPalette(): void {
  paletteOpen.value = true;
}

export function closePalette(): void {
  paletteOpen.value = false;
}

/**
 * Register a command group. Inside a component's setup the registration is
 * removed on unmount; the returned function unregisters manually.
 */
export function registerCommands(group: CommandGroupDef): () => void {
  groups.set(group.id, group);
  const unregister = (): void => {
    groups.delete(group.id);
  };
  onScopeDispose(unregister, true);
  return unregister;
}

/** Visible groups (non-empty after `when` filtering), sorted by order. */
export const visibleGroups = computed<CommandGroupDef[]>(() =>
  [...groups.values()]
    .map((group) => {
      const actions = group.actions.filter((action) => action.when === undefined || action.when());
      return { id: group.id, label: group.label, order: group.order ?? 100, actions };
    })
    .filter((group) => group.actions.length > 0)
    .toSorted((a, b) => a.order - b.order),
);
