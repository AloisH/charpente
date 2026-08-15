<script setup lang="ts">
// SSR head from @nuxtjs/i18n: lang/dir on <html>, hreflang alternates,
// og:locale — fed to useHead as-is so it stays reactive to locale switches.
const head = useLocaleHead();
useHead(() => ({
  htmlAttrs: head.value.htmlAttrs,
  link: head.value.link,
  meta: head.value.meta,
}));
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
