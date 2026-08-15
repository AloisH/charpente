// Keyboard shortcuts, wrapped in one composable on purpose: TanStack Hotkeys
// is alpha, so an upstream API break is a one-file fix here instead of a
// codebase-wide one.
import { useHotkey } from "@tanstack/vue-hotkeys";

export interface ShortcutOptions {
  /** Fire even when an input/textarea has focus (default false). */
  inInputs?: boolean;
}

/**
 * `useShortcut("mod+k", open)` — "mod" is ⌘ on macOS, Ctrl elsewhere.
 * Automatically unbinds when the component unmounts.
 */
export function useShortcut(
  combo: string,
  handler: () => void,
  options: ShortcutOptions = {},
): void {
  useHotkey(combo, () => handler(), {
    enableOnFormTags: options.inInputs ?? false,
  });
}
