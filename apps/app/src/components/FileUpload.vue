<script setup lang="ts">
// Drop zone + progress for the presigned upload flow.
import { ref } from "vue";
import { toast } from "vue-sonner";
import { useI18n } from "vue-i18n";

import { problemMessage } from "@/lib/api-errors";
import { useUpload } from "@/composables/useUpload";

const { t } = useI18n();
const { upload, phase, progress } = useUpload();
const inputRef = ref<HTMLInputElement | null>(null);
const dragging = ref(false);

async function handleFile(file: File | undefined): Promise<void> {
  if (file === undefined) return;
  const result = await upload(file);
  if (result === null) {
    toast.error(problemMessage(undefined));
  } else {
    toast.success(t("uploads.uploaded"));
  }
}

function onDrop(event: DragEvent): void {
  dragging.value = false;
  void handleFile(event.dataTransfer?.files[0]);
}

function onPick(event: Event): void {
  const input = event.target as HTMLInputElement;
  void handleFile(input.files?.[0]);
  input.value = "";
}
</script>

<template>
  <div>
    <button
      type="button"
      class="flex w-full cursor-pointer flex-col items-center justify-center rounded-lg border-2 border-dashed border-border p-10 text-muted-foreground transition-colors hover:border-ring"
      :class="{ 'border-ring bg-accent': dragging }"
      @click="inputRef?.click()"
      @dragover.prevent="dragging = true"
      @dragleave="dragging = false"
      @drop.prevent="onDrop"
    >
      <span v-if="phase === 'uploading'">
        {{ t("uploads.uploading") }} {{ Math.round(progress * 100) }}%
      </span>
      <span v-else>{{ t("uploads.drop") }}</span>
    </button>
    <input ref="inputRef" type="file" class="hidden" @change="onPick" />
    <div v-if="phase === 'uploading'" class="mt-2 h-1 overflow-hidden rounded bg-secondary">
      <div
        class="h-full bg-primary transition-all"
        :style="{ width: `${Math.round(progress * 100)}%` }"
      />
    </div>
  </div>
</template>
