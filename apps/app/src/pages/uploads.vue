<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { Download } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { downloadUpload, listUploadsOptions } from "@charpente/api-client";

import AppShell from "@/components/AppShell.vue";
import FileUpload from "@/components/FileUpload.vue";
import { Button } from "@/components/ui/button";

definePage({ meta: { requiresAuth: true } });

const { t, d, n } = useI18n();

const uploads = useQuery(listUploadsOptions({ query: { limit: 50 } }));

async function download(id: string): Promise<void> {
  const { data } = await downloadUpload({ path: { id } });
  if (data !== undefined) window.open(data.get_url, "_blank", "noopener");
}

function formatSize(bytes: number): string {
  if (bytes >= 1_048_576) return `${n(bytes / 1_048_576, { maximumFractionDigits: 1 })} MB`;
  if (bytes >= 1024) return `${n(bytes / 1024, { maximumFractionDigits: 0 })} kB`;
  return `${bytes} B`;
}
</script>

<template>
  <AppShell>
    <h1 class="text-2xl font-semibold">{{ t("uploads.title") }}</h1>

    <div class="mt-6 max-w-2xl space-y-6">
      <FileUpload />

      <p v-if="uploads.data.value?.items.length === 0" class="text-muted-foreground">
        {{ t("uploads.empty") }}
      </p>
      <ul v-else class="divide-y divide-border rounded-lg border border-border">
        <li
          v-for="item in uploads.data.value?.items ?? []"
          :key="item.id"
          class="flex items-center justify-between gap-4 p-3"
        >
          <div class="min-w-0">
            <p class="truncate font-medium">{{ item.filename }}</p>
            <p class="text-sm text-muted-foreground">
              {{ formatSize(item.size_bytes) }} ·
              {{ d(new Date(item.created_at), "short") }}
            </p>
          </div>
          <Button
            v-if="item.state === 'complete'"
            variant="ghost"
            size="icon"
            :aria-label="t('uploads.download')"
            @click="download(item.id)"
          >
            <Download class="size-4" />
          </Button>
        </li>
      </ul>
    </div>
  </AppShell>
</template>
