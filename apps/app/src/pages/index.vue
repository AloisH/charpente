<script setup lang="ts">
// The SPA has no public landing (that's the Nuxt site) — route straight to
// the dashboard; the auth guard bounces anonymous visitors to /login.
import { onMounted } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();
onMounted(() => {
  void router.replace("/dashboard");
});
</script>

<template>
  <div />
</template>
