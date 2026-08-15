<script setup lang="ts">
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { toast } from "vue-sonner";
import { useI18n } from "vue-i18n";

import { listUsersOptions, listUsersQueryKey, setUserRoleMutation } from "@charpente/api-client";
import type { UserDto } from "@charpente/api-client";

import AppShell from "@/components/AppShell.vue";
import { Button } from "@/components/ui/button";
import { problemMessage } from "@/lib/api-errors";

definePage({ meta: { requiresAuth: true, requiresRole: "admin" } });

const { t, d } = useI18n();
const queryClient = useQueryClient();

const users = useQuery(listUsersOptions({ query: { limit: 50 } }));

const setRole = useMutation({
  ...setUserRoleMutation(),
  onSuccess: async () => {
    toast.success(t("admin.roleUpdated"));
    await queryClient.invalidateQueries({ queryKey: listUsersQueryKey({ query: { limit: 50 } }) });
  },
  onError: (error) => toast.error(problemMessage(error)),
});

function toggleRole(user: UserDto): void {
  setRole.mutate({
    path: { id: user.id },
    body: { role: user.role === "admin" ? "user" : "admin" },
  });
}
</script>

<template>
  <AppShell>
    <h1 class="text-2xl font-semibold">{{ t("admin.usersTitle") }}</h1>

    <div class="mt-6 overflow-x-auto rounded-lg border border-border">
      <table class="w-full text-left text-sm">
        <thead class="border-b border-border bg-muted/50">
          <tr>
            <th class="p-3 font-medium">{{ t("auth.displayName") }}</th>
            <th class="p-3 font-medium">{{ t("auth.email") }}</th>
            <th class="p-3 font-medium">{{ t("admin.role") }}</th>
            <th class="p-3" />
          </tr>
        </thead>
        <tbody class="divide-y divide-border">
          <tr v-for="user in users.data.value?.items ?? []" :key="user.id">
            <td class="p-3">{{ user.display_name }}</td>
            <td class="p-3 text-muted-foreground">{{ user.email }}</td>
            <td class="p-3">
              <span
                class="rounded px-2 py-0.5 text-xs font-medium"
                :class="
                  user.role === 'admin'
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-secondary text-secondary-foreground'
                "
              >
                {{ user.role }}
              </span>
              <span class="ml-2 text-xs text-muted-foreground">
                {{ d(new Date(user.created_at), "short") }}
              </span>
            </td>
            <td class="p-3 text-right">
              <Button
                variant="outline"
                size="sm"
                :disabled="setRole.isPending.value"
                @click="toggleRole(user)"
              >
                {{ user.role === "admin" ? t("admin.makeUser") : t("admin.makeAdmin") }}
              </Button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </AppShell>
</template>
