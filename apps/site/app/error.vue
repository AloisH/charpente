<script setup lang="ts">
// Branded, localized error page — the Nuxt default is neither.
import type { NuxtError } from "#app";

const props = defineProps<{ error: NuxtError }>();
const { t } = useI18n();
const localePath = useLocalePath();

const message = computed(() =>
  props.error.statusCode === 404 ? t("error.notFound") : t("error.other"),
);

function goHome(): void {
  void clearError({ redirect: localePath("/") });
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center gap-6 p-6 text-center">
    <p class="font-mono text-sm text-muted-foreground">{{ error.statusCode }}</p>
    <h1 class="text-2xl font-semibold tracking-tight">{{ message }}</h1>
    <button
      type="button"
      class="rounded-lg bg-primary px-5 py-2.5 font-medium text-primary-foreground hover:opacity-90"
      @click="goHome"
    >
      {{ t("error.home") }}
    </button>
  </div>
</template>
