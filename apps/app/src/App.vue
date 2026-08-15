<script setup lang="ts">
import { Toaster } from "@/components/ui/sonner";
import AppErrorBoundary from "@/components/AppErrorBoundary.vue";
</script>

<template>
  <AppErrorBoundary>
    <RouterView />
  </AppErrorBoundary>
  <Toaster position="top-right" />
</template>
