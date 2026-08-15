<script setup lang="ts">
// Shown on authenticated pages until the email is verified. Non-blocking by
// design: the account works, this only nudges.
import { Loader2, MailWarning } from "@lucide/vue";
import { useMutation } from "@tanstack/vue-query";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { resendVerificationMutation } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { useAuth } from "@/composables/useAuth";
import { problemMessage } from "@/lib/api-errors";

const { t } = useI18n();
const { user } = useAuth();

const resend = useMutation({
  ...resendVerificationMutation(),
  onSuccess: () => toast.success(t("verifyEmail.resent")),
  onError: (error) => toast.error(problemMessage(error)),
});
</script>

<template>
  <div
    v-if="user !== null && !user.email_verified"
    class="flex items-center gap-3 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-sm"
    role="status"
  >
    <MailWarning class="size-4 shrink-0 text-amber-600 dark:text-amber-400" />
    <span class="flex-1">{{ t("verifyEmail.banner") }}</span>
    <Button
      variant="outline"
      size="sm"
      :disabled="resend.isPending.value"
      @click="resend.mutate({})"
    >
      <Loader2 v-if="resend.isPending.value" class="size-4 animate-spin" />
      {{ t("verifyEmail.resend") }}
    </Button>
  </div>
</template>
