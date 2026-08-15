<script setup lang="ts">
// Sidebar-footer tile shown while an admin wears someone else's account —
// same visual language as the dev-environment tile just below it, violet so
// the two never read as one thing. The X is the one-click way out.
import { Loader2, VenetianMask, X } from "@lucide/vue";
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { useRouter } from "vue-router";

import { meQueryKey, stopImpersonationMutation } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { useAuth } from "@/composables/useAuth";
import { problemMessage } from "@/lib/api-errors";

const { t } = useI18n();
const { user } = useAuth();
const queryClient = useQueryClient();
const router = useRouter();

const stop = useMutation({
  ...stopImpersonationMutation(),
  onSuccess: async (admin) => {
    // Everything cached belongs to the impersonated user — drop it all.
    queryClient.clear();
    queryClient.setQueryData(meQueryKey(), admin);
    toast.success(t("impersonation.stopped"));
    await router.push("/admin/users");
  },
  onError: (error) => toast.error(problemMessage(error)),
});
</script>

<template>
  <div
    v-if="user?.impersonating === true"
    role="alert"
    :title="`${user.display_name} (${user.email})`"
    class="relative flex items-center gap-2.5 overflow-hidden rounded-lg border border-violet-500/40 bg-gradient-to-r from-violet-500/15 via-fuchsia-500/10 to-violet-500/15 px-2.5 py-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0"
  >
    <span
      class="flex size-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-violet-500 to-fuchsia-600 text-white shadow-sm"
    >
      <VenetianMask class="size-4" />
    </span>
    <div class="grid min-w-0 flex-1 leading-tight group-data-[collapsible=icon]:hidden">
      <span class="text-xs font-semibold text-violet-700 dark:text-violet-300">
        {{ t("impersonation.title") }}
      </span>
      <span class="truncate text-[11px] text-muted-foreground">{{ user.display_name }}</span>
    </div>
    <Button
      variant="ghost"
      size="icon"
      class="size-7 shrink-0 group-data-[collapsible=icon]:hidden"
      :aria-label="t('impersonation.stop')"
      :title="t('impersonation.stop')"
      :disabled="stop.isPending.value"
      @click="stop.mutate({})"
    >
      <Loader2 v-if="stop.isPending.value" class="size-4 animate-spin" />
      <X v-else class="size-4" />
    </Button>
  </div>
</template>
