// Client-side wide events (see the backend's telemetry module). One event per
// failure, carrying the request id from the response header so a user report
// can be joined to the exact backend log line.

interface ApiErrorLike {
  requestId?: string;
  status?: number;
}

function extractErrorFields(error: unknown): ApiErrorLike & { message: string } {
  if (error instanceof Response) {
    return {
      message: `HTTP ${error.status}`,
      status: error.status,
      requestId: error.headers.get("x-request-id") ?? undefined,
    };
  }
  if (error instanceof Error) return { message: error.message };
  return { message: String(error) };
}

export function logEvent(
  name: string,
  fields: Record<string, string | number | boolean | null>,
  error?: unknown,
): void {
  const errorFields = error === undefined ? {} : extractErrorFields(error);
  const event = { name, ...fields, ...errorFields };

  if (import.meta.env.DEV) {
    console.warn("[wide-event]", event);
    return;
  }

  const { requestId, ...rest } = event;
  void fetch("/api/v1/events", {
    method: "POST",
    headers: { "content-type": "application/json" },
    credentials: "include",
    body: JSON.stringify({ name, request_id: requestId ?? null, fields: rest }),
    keepalive: true,
  }).catch(() => {
    // Telemetry must never break the app.
  });
}
