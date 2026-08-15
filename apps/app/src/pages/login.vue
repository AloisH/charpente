<script setup lang="ts">
import { Loader2 } from "@lucide/vue";
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { defineAsyncComponent } from "vue";
import { useForm } from "vee-validate";
import { toast } from "vue-sonner";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";

import { loginMutation, meQueryKey, zLoginRequest } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { applyFieldErrors, problemMessage } from "@/lib/api-errors";
import { lazyThenEager } from "@/lib/forms";
import { toTypedSchema } from "@/lib/zod-schema";

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const queryClient = useQueryClient();

const { handleSubmit, defineField, errors, setErrors } = useForm({
  validationSchema: toTypedSchema(zLoginRequest),
});
const [email, emailProps] = defineField("email", lazyThenEager);
const [password, passwordProps] = defineField("password", lazyThenEager);

const login = useMutation({
  ...loginMutation(),
  onSuccess: async (user) => {
    queryClient.setQueryData(meQueryKey(), user);
    const redirect = route.query["redirect"];
    await router.push(typeof redirect === "string" ? redirect : "/dashboard");
  },
  onError: (error) => {
    applyFieldErrors(error, setErrors);
    toast.error(problemMessage(error));
  },
});

const onSubmit = handleSubmit((values) => {
  login.mutate({ body: values });
});

// Dev-only quick-login buttons. The ternary is folded at build time, so in
// production the component (and its credentials) is never even emitted as a
// chunk — verified by the bundle scan in CI-facing docs.
const DevQuickLogin = import.meta.env.DEV
  ? defineAsyncComponent(() => import("@/components/DevQuickLogin.vue"))
  : null;
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-sm">
      <CardHeader>
        <CardTitle>{{ t("auth.loginTitle") }}</CardTitle>
      </CardHeader>
      <CardContent>
        <form class="space-y-4" novalidate @submit="onSubmit">
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
              autocomplete="current-password"
              v-bind="passwordProps"
            />
            <p v-if="errors.password" class="text-sm text-destructive">
              {{ errors.password }}
            </p>
          </div>
          <Button type="submit" class="w-full" :disabled="login.isPending.value">
            <Loader2 v-if="login.isPending.value" class="size-4 animate-spin" />
            {{ t("auth.login") }}
          </Button>
        </form>
        <p class="mt-4 text-center text-sm text-muted-foreground">
          {{ t("auth.noAccount") }}
          <RouterLink to="/register" class="underline">{{ t("auth.register") }}</RouterLink>
        </p>

        <component :is="DevQuickLogin" v-if="DevQuickLogin !== null" />
      </CardContent>
    </Card>
  </div>
</template>
