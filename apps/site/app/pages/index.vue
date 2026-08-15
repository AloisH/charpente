<script setup lang="ts">
// The vitrine argues one thesis: types flow one way (Rust → OpenAPI → TS),
// so every section shows the actual artifact — real code, real conventions,
// even the ADR of what was deliberately left out. No icons, no stock claims.
const { t, tm, rt } = useI18n();
const appUrl = useRuntimeConfig().public.appUrl;

useSeoMeta({
  title: () => t("home.title"),
  description: () => t("home.description"),
  ogTitle: () => t("home.headline"),
  ogDescription: () => t("home.description"),
});

const githubUrl = "https://github.com/AloisH/charpente";
const command = "copier copy --trust gh:AloisH/charpente my-project";

const copied = ref(false);
async function copyCommand(): Promise<void> {
  try {
    await navigator.clipboard.writeText(command);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch {
    // Clipboard unavailable (permissions, http) — the text stays selectable.
  }
}

// Real excerpts of the pipeline: the DTO in apps/api/src/routes/auth.rs and
// the client Hey API generates from it.
const rustSnippet = `#[derive(Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    #[schema(format = Email)]
    pub email: String,

    #[validate(length(min = 8))]
    #[schema(min_length = 8)]
    pub password: String,
}`;

const tsSnippet = `export const zRegisterRequest = z.object({
  email: z.email(),
  password: z.string().min(8),
});

// + fetch SDK + TanStack Query helpers
registerMutation(); // typed end to end`;

const conventionKeys = [
  "errors",
  "pagination",
  "auth",
  "uploads",
  "events",
  "permissions",
] as const;

const artifacts: Record<(typeof conventionKeys)[number], string> = {
  errors: '{ "code": "validation_failed" }',
  pagination: "?cursor= → { items, next_cursor }",
  auth: "argon2id · m=19456 t=2 p=1",
  uploads: "PUT (presigned) → s3://…",
  events: "x-request-id: 0198f3a1-…",
  permissions: '<Can permission="manage-users">',
};

type CompiledMessage = Parameters<typeof rt>[0];
const prGates = computed(() => (tm("gates.pr") as CompiledMessage[]).map((item) => rt(item)));
const nightGates = computed(() => (tm("gates.night") as CompiledMessage[]).map((item) => rt(item)));

type ScopeRow = Record<"what" | "why" | "cost", CompiledMessage>;
const scopeRows = computed(() =>
  (tm("scope.rows") as ScopeRow[]).map((row) => ({
    what: rt(row.what),
    why: rt(row.why),
    cost: rt(row.cost),
  })),
);
</script>

<template>
  <div>
    <!-- ── Hero ─────────────────────────────────────────────── -->
    <section class="mx-auto w-full max-w-5xl px-6 pt-20 pb-16 sm:pt-28">
      <p class="font-mono text-xs tracking-[0.2em] text-muted-foreground uppercase">
        {{ t("home.eyebrow") }}
      </p>
      <h1 class="mt-5 max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-6xl">
        {{ t("home.headline") }}
      </h1>
      <p class="mt-6 max-w-2xl text-lg text-muted-foreground">
        {{ t("home.subheadline") }}
      </p>

      <div class="mt-10 flex max-w-2xl flex-col gap-3">
        <div
          class="flex items-center justify-between gap-4 rounded-lg border border-border bg-muted/50 py-3 pr-2 pl-4"
        >
          <code class="overflow-x-auto font-mono text-sm whitespace-nowrap">
            <span class="text-muted-foreground select-none">$ </span>{{ command }}
          </code>
          <button
            type="button"
            class="shrink-0 rounded-md border border-border bg-background px-3 py-1.5 font-mono text-xs hover:bg-accent"
            :aria-label="t('home.copy')"
            @click="copyCommand"
          >
            {{ copied ? t("home.copied") : t("home.copy") }}
          </button>
        </div>
        <div class="flex flex-wrap gap-3">
          <a
            :href="appUrl"
            class="inline-flex items-center rounded-lg bg-primary px-5 py-2.5 font-medium text-primary-foreground hover:opacity-90"
          >
            {{ t("home.cta") }}
          </a>
          <a
            :href="githubUrl"
            rel="noopener"
            class="inline-flex items-center rounded-lg border border-border px-5 py-2.5 font-medium hover:bg-accent"
          >
            {{ t("home.ctaGithub") }}
          </a>
        </div>
      </div>
    </section>

    <!-- ── Signature: the one-way type flow ─────────────────── -->
    <section class="border-t border-border">
      <div class="mx-auto w-full max-w-5xl px-6 py-20">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{{ t("flow.title") }}</h2>
        <p class="mt-4 max-w-2xl text-muted-foreground">{{ t("flow.body") }}</p>

        <div class="mt-10 grid items-stretch gap-4 lg:grid-cols-[1fr_auto_1fr]">
          <figure class="flow-stage min-w-0 rounded-lg border border-border bg-card">
            <figcaption
              class="border-b border-border px-4 py-2 font-mono text-xs text-muted-foreground"
            >
              {{ t("flow.rustLabel") }}
            </figcaption>
            <pre
              class="overflow-x-auto p-4 font-mono text-xs leading-relaxed sm:text-sm"
            ><code>{{ rustSnippet }}</code></pre>
          </figure>

          <div
            class="flow-stage flex items-center justify-center gap-3 font-mono text-xs text-muted-foreground lg:flex-col lg:px-2"
            aria-hidden="true"
          >
            <span class="rounded-md border border-border bg-muted/50 px-2.5 py-1">
              openapi.json
            </span>
            <!-- Panels stack on mobile (flow goes down) and sit side by side
                 on lg (flow goes right). -->
            <span class="rotate-90 text-base lg:rotate-0">→</span>
            <span>just gen-api</span>
            <span class="hidden text-[11px] lg:block">{{ t("flow.specLabel") }}</span>
          </div>

          <figure class="flow-stage min-w-0 rounded-lg border border-border bg-card">
            <figcaption
              class="border-b border-border px-4 py-2 font-mono text-xs text-muted-foreground"
            >
              {{ t("flow.tsLabel") }}
            </figcaption>
            <pre
              class="overflow-x-auto p-4 font-mono text-xs leading-relaxed sm:text-sm"
            ><code>{{ tsSnippet }}</code></pre>
          </figure>
        </div>

        <p class="mt-8 max-w-2xl border-l-2 border-foreground pl-4 font-medium">
          {{ t("flow.kicker") }}
        </p>
      </div>
    </section>

    <!-- ── Decided once ─────────────────────────────────────── -->
    <section class="border-t border-border">
      <div class="mx-auto w-full max-w-5xl px-6 py-20">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{{ t("decided.title") }}</h2>
        <p class="mt-4 max-w-2xl text-muted-foreground">{{ t("decided.intro") }}</p>

        <div class="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <article
            v-for="key in conventionKeys"
            :key="key"
            class="rounded-lg border border-border p-5"
          >
            <code
              class="block overflow-x-auto rounded-md bg-muted/50 px-3 py-2 font-mono text-xs whitespace-nowrap text-muted-foreground"
            >
              {{ artifacts[key] }}
            </code>
            <h3 class="mt-4 font-semibold">{{ t(`decided.${key}.title`) }}</h3>
            <p class="mt-2 text-sm text-muted-foreground">{{ t(`decided.${key}.body`) }}</p>
          </article>
        </div>
      </div>
    </section>

    <!-- ── Quality gates ────────────────────────────────────── -->
    <section class="border-t border-border">
      <div class="mx-auto w-full max-w-5xl px-6 py-20">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{{ t("gates.title") }}</h2>

        <div class="mt-8 space-y-6">
          <div>
            <p class="text-sm text-muted-foreground">{{ t("gates.intro") }}</p>
            <ul class="mt-3 flex flex-wrap gap-2">
              <li
                v-for="gate in prGates"
                :key="gate"
                class="rounded-md border border-border px-3 py-1.5 font-mono text-xs"
              >
                {{ gate }}
              </li>
            </ul>
          </div>
          <div>
            <p class="text-sm text-muted-foreground">{{ t("gates.nightly") }}</p>
            <ul class="mt-3 flex flex-wrap gap-2">
              <li
                v-for="gate in nightGates"
                :key="gate"
                class="rounded-md border border-dashed border-border px-3 py-1.5 font-mono text-xs text-muted-foreground"
              >
                {{ gate }}
              </li>
            </ul>
          </div>
        </div>
      </div>
    </section>

    <!-- ── Deliberately out of scope (ADR-0004) ─────────────── -->
    <section class="border-t border-border">
      <div class="mx-auto w-full max-w-5xl px-6 py-20">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{{ t("scope.title") }}</h2>
        <p class="mt-4 max-w-2xl text-muted-foreground">{{ t("scope.intro") }}</p>

        <div class="mt-10 overflow-hidden rounded-lg border border-border">
          <div
            class="hidden gap-4 border-b border-border bg-muted/50 px-5 py-3 font-mono text-xs tracking-wide text-muted-foreground uppercase md:grid md:grid-cols-3"
          >
            <span>{{ t("scope.colWhat") }}</span>
            <span>{{ t("scope.colWhy") }}</span>
            <span>{{ t("scope.colCost") }}</span>
          </div>
          <div class="divide-y divide-border">
            <div
              v-for="row in scopeRows"
              :key="row.what"
              class="grid gap-1 px-5 py-4 md:grid-cols-3 md:gap-4"
            >
              <span class="font-medium">{{ row.what }}</span>
              <span class="text-sm text-muted-foreground">{{ row.why }}</span>
              <span class="text-sm text-muted-foreground">{{ row.cost }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ── Closing ──────────────────────────────────────────── -->
    <section class="border-t border-border">
      <div class="mx-auto w-full max-w-5xl px-6 py-20">
        <h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">{{ t("closing.title") }}</h2>
        <div
          class="mt-6 inline-flex max-w-full items-center rounded-lg border border-border bg-muted/50 px-4 py-3"
        >
          <code class="overflow-x-auto font-mono text-sm whitespace-nowrap">
            <span class="text-muted-foreground select-none">$ </span>{{ command }}
          </code>
        </div>
        <p class="mt-4 max-w-2xl text-sm text-muted-foreground">{{ t("closing.body") }}</p>
      </div>
    </section>
  </div>
</template>

<style scoped>
/* The one orchestrated moment: the pipeline reveals left → right, once. */
@media (prefers-reduced-motion: no-preference) {
  .flow-stage {
    opacity: 0;
    animation: flow-in 0.5s ease-out forwards;
  }
  .flow-stage:nth-child(2) {
    animation-delay: 0.2s;
  }
  .flow-stage:nth-child(3) {
    animation-delay: 0.4s;
  }
  @keyframes flow-in {
    from {
      opacity: 0;
      transform: translateX(-10px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
}
</style>
