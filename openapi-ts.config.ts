import { defineConfig } from "@hey-api/openapi-ts";

// openapi.json is dumped from the Rust code (just gen-openapi); this config
// turns it into the typed client in packages/api-client/src/generated.
// CI re-runs both steps and fails on any diff (generated-check).
export default defineConfig({
  input: "openapi.json",
  output: {
    path: "packages/api-client/src/generated",
    format: false,
    lint: false,
  },
  plugins: [
    { name: "@hey-api/client-fetch", runtimeConfigPath: "../runtime.ts" },
    "@tanstack/vue-query",
    "zod",
  ],
});
