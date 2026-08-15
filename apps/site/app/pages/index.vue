<script setup lang="ts">
const { t } = useI18n();

useSeoMeta({
  title: () => t("home.title"),
  description: () => t("home.description"),
  ogTitle: () => t("home.title"),
  ogDescription: () => t("home.description"),
});
</script>

<template>
  <section class="mx-auto w-full max-w-5xl px-6 py-24 text-center">
    <h1 class="text-4xl font-bold tracking-tight sm:text-5xl">
      {{ t("home.headline") }}
    </h1>
    <p class="mx-auto mt-6 max-w-2xl text-lg text-muted-foreground">
      {{ t("home.subheadline") }}
    </p>
    <div class="mt-10">
      <a
        href="/app"
        class="inline-flex items-center rounded-lg bg-primary px-6 py-3 font-medium text-primary-foreground hover:opacity-90"
      >
        {{ t("home.cta") }}
      </a>
    </div>
  </section>
</template>
