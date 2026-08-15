// Runtime defaults compiled into the generated fetch client
// (referenced from openapi-ts.config.ts via runtimeConfigPath).
import type { CreateClientConfig } from "./generated/client.gen";

/**
 * Request id of the most recent failed API call (from the x-request-id
 * response header). Client-side wide events attach it so a frontend failure
 * can be joined to the exact backend log line.
 */
export let lastErrorRequestId: string | null = null;

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  baseUrl: "/",
  // The session cookie must flow on every request.
  credentials: "include",
  // Non-2xx responses reject, carrying the parsed problem+json.
  throwOnError: true,
  fetch: async (input: RequestInfo | URL, init?: RequestInit) => {
    const response = await fetch(input, init);
    if (!response.ok) {
      lastErrorRequestId = response.headers.get("x-request-id");
    }
    return response;
  },
});
