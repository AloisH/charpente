// Watches openapi.json (re-dumped by cargo-watch on every successful Rust
// build) and reruns Hey API so the typed client stays in sync during `just dev`.
// Debounced, and skipped when the spec hash is unchanged, so the loop stays
// quiet. A failed Rust build never touches openapi.json, so the last good
// client survives.
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, watch } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const specPath = path.join(root, "openapi.json");

let lastHash = "";
let timer = null;
let running = false;

function hashSpec() {
  try {
    return createHash("sha256").update(readFileSync(specPath)).digest("hex");
  } catch {
    return lastHash; // mid-write or missing: treat as unchanged
  }
}

function regenerate() {
  if (running) return;
  const hash = hashSpec();
  if (hash === lastHash) return;
  running = true;
  execFile("pnpm", ["exec", "openapi-ts"], { cwd: root }, (error, _stdout, stderr) => {
    running = false;
    if (error) {
      console.error("[watch-openapi] generation failed:\n" + stderr);
      return;
    }
    lastHash = hash;
    console.log("[watch-openapi] api-client regenerated");
    regenerate(); // pick up a change that arrived mid-run
  });
}

if (!existsSync(specPath)) {
  console.error("[watch-openapi] openapi.json not found — run `just gen-openapi` first");
  process.exit(1);
}

lastHash = ""; // force one generation at startup so the client is fresh
regenerate();

watch(specPath, () => {
  clearTimeout(timer);
  timer = setTimeout(regenerate, 300);
});

console.log("[watch-openapi] watching openapi.json");
