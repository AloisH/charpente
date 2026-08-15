import { defineConfig, devices } from "@playwright/test";

// In CI the target is the production image started by docker-compose.e2e.yml.
// Locally, run `just infra` + the app with test endpoints:
//   cargo run -p api --features test-endpoints
export default defineConfig({
  testDir: "./tests",
  // One worker on purpose: specs share one database and reset it between
  // tests — parallel workers would truncate each other mid-flight.
  fullyParallel: false,
  workers: 1,
  retries: process.env["CI"] === undefined ? 0 : 1,
  reporter: [["html", { open: "never" }], ["list"]],
  use: {
    baseURL: process.env["E2E_BASE_URL"] ?? "http://localhost:8080",
    trace: "retain-on-failure",
    video: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
