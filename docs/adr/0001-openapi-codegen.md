# ADR 0001 — Types flow one way: Rust → OpenAPI → TypeScript

**Status**: accepted

## Decision

The OpenAPI spec is derived from the Rust code (utoipa; `utoipa-axum` couples
route registration with documentation, so an undocumented handler is an
unrouted handler). The frontend client — TS types, fetch SDK, TanStack Query
helpers, Zod schemas — is generated from that spec by Hey API into
`packages/api-client/src/generated`, which is committed and read-only.

## Why

Nothing is written twice, so the two sides cannot drift. Client validation
(Zod → vee-validate) matches server validation (`validator`) by construction.
CI's `generated-check` re-runs both steps and fails on any diff.

## Trade-offs

Generated code is committed: bigger diffs on API changes, but zero-setup
clones and no codegen inside Docker builds. The dev loop
(`cargo watch` → `openapi.json` → `scripts/watch-openapi.mjs` → Vite HMR)
regenerates automatically; a broken Rust build leaves the last good client in
place.
