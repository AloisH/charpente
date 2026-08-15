import { QueryClient } from "@tanstack/vue-query";

import { logEvent } from "@/lib/log-event";

// One query client with sane defaults: server data is trusted for 30s,
// failures are retried once (never for 4xx), and every failed query/mutation
// emits a wide event that joins up with the backend's request log.
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: (failureCount, error) => {
        if (error instanceof Response && error.status < 500) return false;
        return failureCount < 1;
      },
    },
  },
});

queryClient.getQueryCache().subscribe((event) => {
  if (event.type === "updated" && event.query.state.status === "error") {
    logEvent("query_failed", {
      queryKey: JSON.stringify(event.query.queryKey),
    }, event.query.state.error);
  }
});

queryClient.getMutationCache().subscribe((event) => {
  if (event.type === "updated" && event.mutation?.state.status === "error") {
    logEvent("mutation_failed", {
      mutationKey: JSON.stringify(event.mutation.options.mutationKey ?? null),
    }, event.mutation.state.error);
  }
});
