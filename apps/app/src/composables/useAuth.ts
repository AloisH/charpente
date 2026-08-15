import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed } from "vue";
import { useRouter } from "vue-router";

import { logoutMutation, meOptions } from "@charpente/api-client";
import type { UserDto } from "@charpente/api-client";

/** Permissions the SPA understands, mirroring the backend's Role::permissions. */
const rolePermissions: Record<UserDto["role"], readonly Permission[]> = {
  admin: ["manage-users", "view-audit-log"],
  user: [],
};

export type Permission = "manage-users" | "view-audit-log";

export function useAuth() {
  const queryClient = useQueryClient();
  const router = useRouter();

  // The generated `me` query; a 401 means "not logged in", never a retry.
  const query = useQuery({ ...meOptions(), retry: false, staleTime: 60_000 });
  const user = computed<UserDto | null>(() => query.data.value ?? null);
  const isAuthenticated = computed(() => user.value !== null);

  const can = (permission: Permission): boolean => {
    const current = user.value;
    if (current === null) return false;
    return rolePermissions[current.role].includes(permission);
  };

  const logout = useMutation({
    ...logoutMutation(),
    onSuccess: async () => {
      queryClient.clear();
      await router.push("/login");
    },
  });

  return { user, isAuthenticated, isLoading: query.isLoading, can, logout };
}
