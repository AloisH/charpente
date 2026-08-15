<script setup lang="ts">
// Shown whenever the session is an admin wearing someone else's account —
// unmissable on purpose, with the way out one click away.
import { Loader2, VenetianMask } from "@lucide/vue";
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
    class="flex items-center gap-3 border-b border-violet-500/40 bg-gradient-to-r from-violet-500/15 via-fuchsia-500/10 to-violet-500/15 px-4 py-2 text-sm"
    role="alert"
  >
    <span
      class="flex size-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-violet-500 to-fuchsia-600 text-white shadow-sm"
    >
      <VenetianMask class="size-4" />
    </span>
    <span class="flex-1">
      {{ t("impersonation.banner", { name: user.display_name, email: user.email }) }}
    </span>
    <Button variant="outline" size="sm" :disabled="stop.isPending.value" @click="stop.mutate({})">
      <Loader2 v-if="stop.isPending.value" class="size-4 animate-spin" />
      {{ t("impersonation.stop") }}
    </Button>
  </div>
</template>
