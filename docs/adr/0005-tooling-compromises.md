# ADR 0005 — Tooling compromises worth knowing

**Status**: accepted

- **oxlint only, no ESLint.** oxlint has no Vue _template_ rules
  (`eslint-plugin-vue` equivalents). Compensation: `vue-tsc` strict typecheck
  (typed templates, wrong/missing props), `noUncheckedIndexedAccess`, and axe
  accessibility assertions in e2e. Revisit when oxc ships template linting.
- **oxfmt is pre-1.0** — pinned to an exact version; bump deliberately (its
  output may shift between versions).
- **No `exactOptionalPropertyTypes`.** Tried; reka-ui, vue-sonner and the Hey
  API generated client all fail under it. `noUncheckedIndexedAccess` (the
  higher-value flag) stays.
- **TanStack Hotkeys is alpha** — wrapped in `composables/useShortcut.ts` so an
  API break is a one-file fix.
- **vee-validate's zod adapter targets zod v3**; the generated schemas are zod
  v4, so `src/lib/zod-schema.ts` is a ~30-line local TypedSchema adapter.
  Replace with the official adapter when it supports v4 / standard-schema.
- **RustFS is younger than MinIO.** Contained risk: the code talks plain S3
  (`aws-sdk-s3`, path-style); swap the endpoint env var if it disappoints.
- **`.sqlx` offline data is committed** so Docker builds need no database; CI
  verifies it's current (`cargo sqlx prepare --check`).
