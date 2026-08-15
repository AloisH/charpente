<script setup lang="ts">
import { VueQueryDevtools } from "@tanstack/vue-query-devtools";

import AppErrorBoundary from "@/components/AppErrorBoundary.vue";
import { Toaster } from "@/components/ui/sonner";
</script>

<template>
  <AppErrorBoundary>
    <RouterView />
  </AppErrorBoundary>
  <Toaster position="top-right" />
  <VueQueryDevtools />
</template>
