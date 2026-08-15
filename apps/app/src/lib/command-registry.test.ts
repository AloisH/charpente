import { describe, expect, it } from "vitest";
import { effectScope } from "vue";

import { registerCommands, visibleGroups } from "@/lib/command-registry";

function group(id: string, order?: number, when?: () => boolean) {
  return {
    id,
    label: () => id,
    ...(order === undefined ? {} : { order }),
    actions: [
      {
        id: `${id}-action`,
        label: () => `${id} action`,
        ...(when === undefined ? {} : { when }),
        perform: () => {},
      },
    ],
  };
}

describe("command registry", () => {
  it("registers and manually unregisters", () => {
    const unregister = registerCommands(group("g1"));
    expect(visibleGroups.value.map((g) => g.id)).toContain("g1");
    unregister();
    expect(visibleGroups.value.map((g) => g.id)).not.toContain("g1");
  });

  it("auto-unregisters when the owning scope is disposed (page unmount)", () => {
    const scope = effectScope();
    scope.run(() => {
      registerCommands(group("page-scoped"));
    });
    expect(visibleGroups.value.map((g) => g.id)).toContain("page-scoped");
    scope.stop();
    expect(visibleGroups.value.map((g) => g.id)).not.toContain("page-scoped");
  });

  it("sorts by order and hides groups whose actions are all filtered out", () => {
    const cleanups = [
      registerCommands(group("later", 50)),
      registerCommands(group("first", 1)),
      registerCommands(group("hidden", 2, () => false)),
    ];
    const ids = visibleGroups.value.map((g) => g.id);
    expect(ids).toEqual(["first", "later"]);
    for (const cleanup of cleanups) cleanup();
  });
});
