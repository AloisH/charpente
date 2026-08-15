<script setup lang="ts">
import { Loader2, MailCheck } from "@lucide/vue";
import { useMutation } from "@tanstack/vue-query";
import { useForm } from "vee-validate";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { forgotPasswordMutation, zForgotPasswordRequest } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { applyFieldErrors, problemMessage } from "@/lib/api-errors";
import { lazyThenEager } from "@/lib/forms";
import { toTypedSchema } from "@/lib/zod-schema";

const { t } = useI18n();

const { handleSubmit, defineField, errors, setErrors } = useForm({
  validationSchema: toTypedSchema(zForgotPasswordRequest),
});
const [email, emailProps] = defineField("email", lazyThenEager);

const forgot = useMutation({
  ...forgotPasswordMutation(),
  onError: (error) => {
    applyFieldErrors(error, setErrors);
    toast.error(problemMessage(error));
  },
});

const onSubmit = handleSubmit((values) => {
  forgot.mutate({ body: values });
});
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-sm">
      <CardHeader>
        <CardTitle>{{ t("resetPassword.forgotTitle") }}</CardTitle>
      </CardHeader>
      <CardContent>
        <!-- Always the same success message: whether the account exists is
             never revealed, mirroring the API's constant 204. -->
        <div v-if="forgot.isSuccess.value" class="flex items-start gap-2 text-sm">
          <MailCheck class="mt-0.5 size-4 shrink-0 text-green-600 dark:text-green-400" />
          {{ t("resetPassword.sent") }}
        </div>

        <form v-else class="space-y-4" novalidate @submit="onSubmit">
          <p class="text-sm text-muted-foreground">{{ t("resetPassword.forgotHint") }}</p>
          <div class="space-y-2">
            <Label for="email">{{ t("auth.email") }}</Label>
            <Input
              id="email"
              v-model="email"
              type="email"
              autocomplete="email"
              v-bind="emailProps"
            />
            <p v-if="errors.email" class="text-sm text-destructive">{{ errors.email }}</p>
          </div>
          <Button type="submit" class="w-full" :disabled="forgot.isPending.value">
            <Loader2 v-if="forgot.isPending.value" class="size-4 animate-spin" />
            {{ t("resetPassword.sendLink") }}
          </Button>
        </form>

        <p class="mt-4 text-center text-sm text-muted-foreground">
          <RouterLink to="/login" class="underline">{{
            t("resetPassword.backToLogin")
          }}</RouterLink>
        </p>
      </CardContent>
    </Card>
  </div>
</template>
