<script setup lang="ts">
// Landing page for the emailed link. Public on purpose: the link may be
// opened in a browser without a session; verifying needs only the token.
import { CircleCheck, CircleX, Loader2 } from "@lucide/vue";
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

import { meQueryKey, verifyEmailMutation } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const { t } = useI18n();
const route = useRoute();
const queryClient = useQueryClient();

const verify = useMutation({
  ...verifyEmailMutation(),
  onSuccess: async () => {
    // A logged-in session sees the banner disappear immediately.
    await queryClient.invalidateQueries({ queryKey: meQueryKey() });
  },
});

const token = typeof route.query["token"] === "string" ? route.query["token"] : "";

onMounted(() => {
  if (token !== "") {
    verify.mutate({ body: { token } });
  }
});
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-sm">
      <CardHeader>
        <CardTitle>{{ t("verifyEmail.title") }}</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <!-- Idle counts as "verifying": the mutation fires on mount, so the
             error branch must not flash during the first render. -->
        <div
          v-if="token !== '' && !verify.isSuccess.value && !verify.isError.value"
          class="flex items-center gap-2 text-sm"
        >
          <Loader2 class="size-4 animate-spin" />
          {{ t("verifyEmail.verifying") }}
        </div>

        <template v-else-if="verify.isSuccess.value">
          <div class="flex items-center gap-2 text-sm">
            <CircleCheck class="size-4 text-green-600 dark:text-green-400" />
            {{ t("verifyEmail.success") }}
          </div>
          <Button as-child class="w-full">
            <RouterLink to="/dashboard">{{ t("verifyEmail.goToDashboard") }}</RouterLink>
          </Button>
        </template>

        <template v-else>
          <div class="flex items-center gap-2 text-sm">
            <CircleX class="size-4 text-destructive" />
            {{ t("verifyEmail.invalid") }}
          </div>
          <Button as-child variant="outline" class="w-full">
            <RouterLink to="/dashboard">{{ t("verifyEmail.goToDashboard") }}</RouterLink>
          </Button>
        </template>
      </CardContent>
    </Card>
  </div>
</template>
