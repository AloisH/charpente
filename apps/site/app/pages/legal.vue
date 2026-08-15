<script setup lang="ts">
const { t } = useI18n();

useSeoMeta({
  title: () => t("legal.title"),
  robots: "noindex",
});
</script>

<template>
  <section class="mx-auto w-full max-w-3xl px-6 py-16">
    <h1 class="text-3xl font-bold">{{ t("legal.title") }}</h1>
    <p class="mt-6 text-muted-foreground">{{ t("legal.placeholder") }}</p>

    <h2 class="mt-10 text-xl font-semibold">{{ t("legal.privacyTitle") }}</h2>
    <p class="mt-4 text-muted-foreground">{{ t("legal.privacyPlaceholder") }}</p>
  </section>
</template>
