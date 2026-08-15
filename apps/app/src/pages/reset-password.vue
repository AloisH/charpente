<script setup lang="ts">
// Landing page for the reset link. Public: the token is the credential.
import { Loader2 } from "@lucide/vue";
import { useMutation } from "@tanstack/vue-query";
import { useForm } from "vee-validate";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";
import { useRoute, useRouter } from "vue-router";

import { resetPasswordMutation, zResetPasswordRequest } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { applyFieldErrors, problemMessage } from "@/lib/api-errors";
import { lazyThenEager } from "@/lib/forms";
import { toTypedSchema } from "@/lib/zod-schema";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const token = typeof route.query["token"] === "string" ? route.query["token"] : "";

const { handleSubmit, defineField, errors, setErrors } = useForm({
  validationSchema: toTypedSchema(zResetPasswordRequest.pick({ password: true })),
});
const [password, passwordProps] = defineField("password", lazyThenEager);

const reset = useMutation({
  ...resetPasswordMutation(),
  onSuccess: async () => {
    toast.success(t("resetPassword.done"));
    await router.push("/login");
  },
  onError: (error) => {
    applyFieldErrors(error, setErrors);
    // A 404 means the token is dead — say so instead of a generic error.
    toast.error(
      (error as { status?: number }).status === 404
        ? t("resetPassword.invalid")
        : problemMessage(error),
    );
  },
});

const onSubmit = handleSubmit((values) => {
  reset.mutate({ body: { token, password: values.password } });
});
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-sm">
      <CardHeader>
        <CardTitle>{{ t("resetPassword.title") }}</CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="token === ''" class="space-y-4 text-sm">
          <p>{{ t("resetPassword.invalid") }}</p>
          <Button as-child variant="outline" class="w-full">
            <RouterLink to="/forgot-password">{{ t("resetPassword.requestNew") }}</RouterLink>
          </Button>
        </div>

        <form v-else class="space-y-4" novalidate @submit="onSubmit">
          <div class="space-y-2">
            <Label for="password">{{ t("resetPassword.newPassword") }}</Label>
            <Input
              id="password"
              v-model="password"
              type="password"
              autocomplete="new-password"
              v-bind="passwordProps"
            />
            <p v-if="errors.password" class="text-sm text-destructive">
              {{ errors.password }}
            </p>
          </div>
          <Button type="submit" class="w-full" :disabled="reset.isPending.value">
            <Loader2 v-if="reset.isPending.value" class="size-4 animate-spin" />
            {{ t("resetPassword.submit") }}
          </Button>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
