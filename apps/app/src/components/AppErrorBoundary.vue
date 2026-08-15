<script setup lang="ts">
// Global error boundary: a rendering error anywhere below shows a recoverable
// screen instead of a blank page, and emits one wide event.
import { onErrorCaptured, ref } from "vue";
import { useI18n } from "vue-i18n";

import { Button } from "@/components/ui/button";
import { logEvent } from "@/lib/log-event";

const { t } = useI18n();
const failed = ref(false);

onErrorCaptured((error) => {
  failed.value = true;
  logEvent("render_error", {}, error);
  return false;
});

function reload(): void {
  window.location.reload();
}
</script>

<template>
  <div
    v-if="failed"
    class="flex min-h-screen flex-col items-center justify-center gap-4 p-8 text-center"
  >
    <h1 class="text-2xl font-semibold">{{ t("common.error") }}</h1>
    <Button @click="reload">{{ t("common.retry") }}</Button>
  </div>
  <slot v-else />
</template>
