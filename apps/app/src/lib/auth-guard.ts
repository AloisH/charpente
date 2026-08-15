import type { Router } from "vue-router";

import { meOptions } from "@charpente/api-client";
import type { UserDto } from "@charpente/api-client";

import { queryClient } from "@/lib/query";

// Route-level auth. Pages opt in via definePage:
//   definePage({ meta: { requiresAuth: true } })
//   definePage({ meta: { requiresAuth: true, requiresRole: "admin" } })
export function installAuthGuard(router: Router): void {
  router.beforeEach(async (to) => {
    if (to.meta.requiresAuth !== true) return true;

    const user: UserDto | null = await queryClient
      .ensureQueryData({ ...meOptions(), retry: false, staleTime: 60_000 })
      .catch(() => null);

    if (user === null) {
      return { path: "/login", query: { redirect: to.fullPath } };
    }
    if (to.meta.requiresRole !== undefined && user.role !== to.meta.requiresRole) {
      return { path: "/" };
    }
    return true;
  });
}

declare module "vue-router" {
  interface RouteMeta {
    requiresAuth?: boolean;
    requiresRole?: "admin" | "user";
  }
}
