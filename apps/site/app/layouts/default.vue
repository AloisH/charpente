<script setup lang="ts">
const { t } = useI18n();
const localePath = useLocalePath();
const switchLocalePath = useSwitchLocalePath();
const { locale } = useI18n();
const otherLocale = computed(() => (locale.value === "fr" ? "en" : "fr"));
</script>

<template>
  <div class="flex min-h-screen flex-col">
    <header class="border-b border-border">
      <nav
        class="mx-auto flex w-full max-w-5xl items-center justify-between px-6 py-4"
        :aria-label="t('nav.main')"
      >
        <NuxtLink :to="localePath('/')" class="text-lg font-semibold"> charpente </NuxtLink>
        <div class="flex items-center gap-6 text-sm">
          <a href="/app" class="font-medium hover:underline">{{ t("nav.app") }}</a>
          <NuxtLink
            :to="switchLocalePath(otherLocale)"
            class="text-muted-foreground hover:underline"
          >
            {{ otherLocale.toUpperCase() }}
          </NuxtLink>
        </div>
      </nav>
    </header>

    <main class="flex-1">
      <slot />
    </main>

    <footer class="border-t border-border">
      <div
        class="mx-auto flex w-full max-w-5xl items-center justify-between px-6 py-6 text-sm text-muted-foreground"
      >
        <span>© {{ new Date().getFullYear() }} charpente</span>
        <NuxtLink :to="localePath('/legal')" class="hover:underline">
          {{ t("nav.legal") }}
        </NuxtLink>
      </div>
    </footer>
  </div>
</template>
