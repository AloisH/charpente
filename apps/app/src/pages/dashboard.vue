<script setup lang="ts">
import { Upload } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import AppShell from "@/components/AppShell.vue";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuth } from "@/composables/useAuth";
import { registerCommands } from "@/lib/command-registry";

definePage({ meta: { requiresAuth: true } });

const { t } = useI18n();
const router = useRouter();
const { user } = useAuth();

// Page-scoped palette commands: registered while this page is mounted,
// gone when it unmounts — the pattern every feature page can copy.
registerCommands({
  id: "dashboard-quick-actions",
  label: () => t("command.quickActions"),
  actions: [
    {
      id: "upload",
      label: () => t("command.goUpload"),
      icon: Upload,
      perform: () => router.push("/uploads"),
    },
  ],
});
</script>

<template>
  <AppShell>
    <h1 class="text-2xl font-semibold">{{ t("dashboard.title") }}</h1>
    <Card class="mt-6">
      <CardHeader>
        <CardTitle>
          {{ t("dashboard.welcome", { name: user?.display_name ?? "…" }) }}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <p class="text-muted-foreground">{{ t("dashboard.hint") }}</p>
      </CardContent>
    </Card>
  </AppShell>
</template>
