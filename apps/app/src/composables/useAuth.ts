import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed } from "vue";
import { useRouter } from "vue-router";

import { logoutMutation, meOptions } from "@charpente/api-client";
import type { UserDto } from "@charpente/api-client";

// The generated `me` query, with 401 treated as "not logged in", not an error.
export function currentUserQueryOptions() {
  return {
    ...meOptions(),
    retry: false,
    staleTime: 60_000,
  };
}

/** Permissions the SPA understands, mirroring the backend's Role::permissions. */
const rolePermissions: Record<UserDto["role"], readonly string[]> = {
  admin: ["manage-users", "view-audit-log"],
  user: [],
};

export function useAuth() {
  const queryClient = useQueryClient();
  const router = useRouter();

  const query = useQuery(currentUserQueryOptions());
  const user = computed<UserDto | null>(() => query.data.value ?? null);
  const isAuthenticated = computed(() => user.value !== null);

  const can = (permission: string): boolean => {
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
