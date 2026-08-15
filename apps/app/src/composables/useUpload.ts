// The presigned upload flow as one composable:
//   declare → PUT to storage (with progress) → confirm → refresh the list.
import { useQueryClient } from "@tanstack/vue-query";
import { ref } from "vue";

import { completeUpload, createUpload, listUploadsQueryKey } from "@charpente/api-client";
import type { UploadDto } from "@charpente/api-client";

export type UploadPhase = "idle" | "requesting" | "uploading" | "confirming" | "done" | "error";

export function useUpload() {
  const queryClient = useQueryClient();
  const phase = ref<UploadPhase>("idle");
  const progress = ref(0);
  const error = ref<unknown>(null);

  async function upload(file: File): Promise<UploadDto | null> {
    phase.value = "requesting";
    progress.value = 0;
    error.value = null;

    try {
      const { data: created } = await createUpload({
        body: {
          filename: file.name,
          content_type: file.type,
          size_bytes: file.size,
        },
      });
      if (created === undefined) throw new Error("empty create-upload response");

      phase.value = "uploading";
      await putWithProgress(created.put_url, file, (fraction) => {
        progress.value = fraction;
      });

      phase.value = "confirming";
      const { data: completed } = await completeUpload({
        path: { id: created.upload.id },
      });
      if (completed === undefined) throw new Error("empty complete-upload response");

      phase.value = "done";
      await queryClient.invalidateQueries({ queryKey: listUploadsQueryKey() });
      return completed;
    } catch (cause) {
      phase.value = "error";
      error.value = cause;
      return null;
    }
  }

  return { upload, phase, progress, error };
}

// fetch() has no upload progress; XHR does.
function putWithProgress(
  url: string,
  file: File,
  onProgress: (fraction: number) => void,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("PUT", url);
    xhr.setRequestHeader("content-type", file.type);
    xhr.upload.addEventListener("progress", (event) => {
      if (event.lengthComputable) onProgress(event.loaded / event.total);
    });
    xhr.addEventListener("load", () => {
      if (xhr.status >= 200 && xhr.status < 300) resolve();
      else reject(new Error(`storage PUT failed with ${xhr.status}`));
    });
    xhr.addEventListener("error", () => reject(new Error("storage PUT failed")));
    xhr.send(file);
  });
}
