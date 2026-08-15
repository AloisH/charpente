<script setup lang="ts">
import { useI18n } from "vue-i18n";

import AppShell from "@/components/AppShell.vue";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuth } from "@/composables/useAuth";

definePage({ meta: { requiresAuth: true } });

const { t } = useI18n();
const { user } = useAuth();
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
