// Keyboard shortcuts, wrapped in one composable on purpose: TanStack Hotkeys
// is alpha, so an upstream API break is a one-file fix here instead of a
// codebase-wide one.
import { formatForDisplay, type RegisterableHotkey } from "@tanstack/hotkeys";
import { useHotkey } from "@tanstack/vue-hotkeys";

export type Shortcut = RegisterableHotkey;

export interface ShortcutOptions {
  /** Fire even when an input/textarea has focus (default: library heuristic). */
  inInputs?: boolean;
}

/**
 * `useShortcut("Mod+K", open)` — "Mod" is ⌘ on macOS, Ctrl elsewhere.
 * Automatically unbinds when the component unmounts.
 */
export function useShortcut(
  combo: Shortcut,
  handler: () => void,
  options: ShortcutOptions = {},
): void {
  const hotkeyOptions = options.inInputs === undefined ? {} : { ignoreInputs: !options.inInputs };
  useHotkey(combo, () => handler(), hotkeyOptions);
}

/**
 * Platform-aware display string for a shortcut: "⌘K" on macOS,
 * "Ctrl+K" on Windows/Linux. Use for every visible kbd hint.
 */
export function formatShortcut(combo: Shortcut): string {
  return formatForDisplay(combo);
}
