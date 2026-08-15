<script setup lang="ts">
// Sidebar-footer tile shown until the email is verified — same visual
// language as the dev-environment and impersonation tiles, sky-blue so the
// three stay tellable apart. Non-blocking by design: the account works, this
// only nudges. The send button re-mails the verification link.
import { Loader2, MailWarning, Send } from "@lucide/vue";
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
    role="status"
    :title="t('verifyEmail.banner')"
    class="relative flex items-center gap-2.5 overflow-hidden rounded-lg border border-sky-500/40 bg-gradient-to-r from-sky-500/15 via-cyan-500/10 to-sky-500/15 px-2.5 py-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:border-0 group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0"
  >
    <span
      class="flex size-7 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-sky-500 to-cyan-600 text-white shadow-sm"
    >
      <MailWarning class="size-4" />
    </span>
    <div class="grid min-w-0 flex-1 leading-tight group-data-[collapsible=icon]:hidden">
      <span class="text-xs font-semibold text-sky-700 dark:text-sky-300">
        {{ t("verifyEmail.tileTitle") }}
      </span>
      <span class="truncate text-[11px] text-muted-foreground">{{ user.email }}</span>
    </div>
    <Button
      variant="ghost"
      size="icon"
      class="size-7 shrink-0 text-sky-700 hover:bg-sky-500/20 hover:text-sky-800 dark:text-sky-300 dark:hover:bg-sky-400/20 dark:hover:text-sky-200 group-data-[collapsible=icon]:hidden"
      :aria-label="t('verifyEmail.resend')"
      :title="t('verifyEmail.resend')"
      :disabled="resend.isPending.value"
      @click="resend.mutate({})"
    >
      <Loader2 v-if="resend.isPending.value" class="size-4 animate-spin" />
      <Send v-else class="size-4" />
    </Button>
  </div>
</template>
