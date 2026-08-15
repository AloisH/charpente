<script setup lang="ts">
import { Loader2 } from "@lucide/vue";
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { useForm } from "vee-validate";
import { toast } from "vue-sonner";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { meQueryKey, registerMutation, zRegisterRequest } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { applyFieldErrors, problemMessage } from "@/lib/api-errors";
import { toTypedSchema } from "@/lib/zod-schema";

const { t } = useI18n();
const router = useRouter();
const queryClient = useQueryClient();

const { handleSubmit, defineField, errors, setErrors } = useForm({
  validationSchema: toTypedSchema(zRegisterRequest),
});
const [email, emailProps] = defineField("email");
const [password, passwordProps] = defineField("password");
const [displayName, displayNameProps] = defineField("display_name");

const register = useMutation({
  ...registerMutation(),
  onSuccess: async (user) => {
    queryClient.setQueryData(meQueryKey(), user);
    await router.push("/dashboard");
  },
  onError: (error) => {
    applyFieldErrors(error, setErrors);
    toast.error(problemMessage(error));
  },
});

const onSubmit = handleSubmit((values) => {
  register.mutate({ body: values });
});
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-sm">
      <CardHeader>
        <CardTitle>{{ t("auth.registerTitle") }}</CardTitle>
      </CardHeader>
      <CardContent>
        <form class="space-y-4" novalidate @submit="onSubmit">
          <div class="space-y-2">
            <Label for="display_name">{{ t("auth.displayName") }}</Label>
            <Input
              id="display_name"
              v-model="displayName"
              autocomplete="name"
              v-bind="displayNameProps"
            />
            <p v-if="errors.display_name" class="text-sm text-destructive">
              {{ errors.display_name }}
            </p>
          </div>
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
          <div class="space-y-2">
            <Label for="password">{{ t("auth.password") }}</Label>
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
          <Button type="submit" class="w-full" :disabled="register.isPending.value">
            <Loader2 v-if="register.isPending.value" class="size-4 animate-spin" />
            {{ t("auth.register") }}
          </Button>
        </form>
        <p class="mt-4 text-center text-sm text-muted-foreground">
          {{ t("auth.haveAccount") }}
          <RouterLink to="/login" class="underline">{{ t("auth.login") }}</RouterLink>
        </p>
      </CardContent>
    </Card>
  </div>
</template>
