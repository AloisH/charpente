// Client-side wide events (see the backend's telemetry module). One event per
// failure, carrying the request id captured by the API client so a user
// report can be joined to the exact backend log line.
import { lastErrorRequestId } from "@charpente/api-client/runtime";

import { isProblem } from "@/lib/api-errors";

export function logEvent(
  name: string,
  fields: Record<string, string | number | boolean | null>,
  error?: unknown,
): void {
  const extra: Record<string, string | number | boolean | null> = { ...fields };
  let requestId: string | null = null;

  if (isProblem(error)) {
    extra["status"] = error.status;
    extra["code"] = error.code;
    requestId = lastErrorRequestId;
  } else if (error instanceof Error) {
    extra["message"] = error.message;
  } else if (error !== undefined) {
    extra["message"] = String(error);
  }

  if (import.meta.env.DEV) {
    console.warn("[wide-event]", name, { ...extra, requestId });
    return;
  }

  void fetch("/api/v1/events", {
    method: "POST",
    headers: { "content-type": "application/json" },
    credentials: "include",
    body: JSON.stringify({ name, request_id: requestId, fields: extra }),
    keepalive: true,
  }).catch(() => {
    // Telemetry must never break the app.
  });
}
