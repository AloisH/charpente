<script setup lang="ts">
// Dev-only quick login: one click into an admin or plain-user session.
// This component is only ever imported behind `import.meta.env.DEV`
// (see login.vue), so neither this code nor the credentials reach a
// production bundle.
import { Loader2 } from "@lucide/vue";
import { useQueryClient } from "@tanstack/vue-query";
import { ref } from "vue";
import { toast } from "vue-sonner";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { login as apiLogin, meQueryKey, register as apiRegister } from "@charpente/api-client";

import { Button } from "@/components/ui/button";
import { problemMessage } from "@/lib/api-errors";

const { t } = useI18n();
const router = useRouter();
const queryClient = useQueryClient();

/** Which button is in flight — both are disabled while one is. */
const pending = ref<"admin" | "user" | null>(null);

async function quickLogin(kind: "admin" | "user"): Promise<void> {
  if (pending.value !== null) return;
  pending.value = kind;

  const credentials =
    kind === "admin"
      ? {
          email: import.meta.env.VITE_SEED_ADMIN_EMAIL ?? "admin@example.com",
          password: import.meta.env.VITE_SEED_ADMIN_PASSWORD ?? "changeme-admin-password",
        }
      : { email: "dev-user@example.com", password: "dev-user-password" };

  try {
    let user;
    try {
      ({ data: user } = await apiLogin({ body: credentials }));
    } catch (error) {
      // The throwaway dev user self-registers on first use.
      if (kind !== "user") throw error;
      ({ data: user } = await apiRegister({
        body: { ...credentials, display_name: "Dev User" },
      }));
    }
    queryClient.setQueryData(meQueryKey(), user);
    await router.push("/dashboard");
  } catch (error) {
    toast.error(problemMessage(error));
  } finally {
    pending.value = null;
  }
}
</script>

<template>
  <div class="mt-6 border-t border-dashed border-border pt-4">
    <p class="mb-2 text-center text-xs uppercase tracking-wide text-muted-foreground">
      {{ t("auth.devLogin") }}
    </p>
    <div class="flex gap-2">
      <Button
        variant="outline"
        size="sm"
        class="flex-1"
        :disabled="pending !== null"
        @click="quickLogin('admin')"
      >
        <Loader2 v-if="pending === 'admin'" class="size-4 animate-spin" />
        {{ t("auth.devLoginAdmin") }}
      </Button>
      <Button
        variant="outline"
        size="sm"
        class="flex-1"
        :disabled="pending !== null"
        @click="quickLogin('user')"
      >
        <Loader2 v-if="pending === 'user'" class="size-4 animate-spin" />
        {{ t("auth.devLoginUser") }}
      </Button>
    </div>
  </div>
</template>
