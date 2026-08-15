# Agent guide

Monorepo: Rust/Axum API (`apps/api`), Vue 3 SPA (`apps/app`), Nuxt site
(`apps/site`), generated API client (`packages/api-client`), shared design
tokens (`packages/tokens`), Playwright suite (`e2e/`).

## Commands

```sh
just infra          # start postgres/rustfs/adminer/mailpit (needed for sqlx macros)
just lint           # every formatter/linter/typecheck — run before committing
just test           # cargo tests + vitest
just gen-api        # after changing any Rust DTO/handler: re-dump spec + regen client
just sqlx-prepare   # after changing any sqlx query: refresh committed .sqlx
just migrate-new <name>
```

`cargo` commands need `DATABASE_URL` (see `.env.example`) unless
`SQLX_OFFLINE=1`.

## Hard rules

- **Never edit `packages/api-client/src/generated/`** — it is produced from
  `openapi.json` by `just gen-api`; CI fails on drift. Change the Rust side
  instead.
- **Never edit `openapi.json`** by hand — it is dumped from the utoipa
  annotations in `apps/api`.
- New/changed sqlx queries require `just sqlx-prepare` (commit the `.sqlx`
  diff) or CI fails.
- Clippy denies `unwrap`/`expect`/`panic`/`todo` outside tests. Return
  `AppError` (problem+json) from handlers; attach context with `anyhow`.
- New API routes: annotate with `#[utoipa::path]`, register via
  `routes!(handler)` in the module's `router()`, keep them under `/api/v1`.
  List endpoints use `CursorParams` → `Page<T>`.
- Request DTOs: every `#[validate(...)]` rule needs a matching
  `#[schema(...)]` attribute (format = Email, min_length, maximum, …) —
  that is what puts the constraint into the OpenAPI spec and therefore into
  the generated client-side Zod validation.
- User-facing strings in the SPA/site go through i18n — add both `fr` and `en`
  keys (`apps/{app,site}/…/locales/*.json` or `i18n/locales/`).
- Server state lives in TanStack Query via the generated helpers; Pinia is for
  client-only UI state.
- Conventional commits (commitlint enforces this).

## Tests

- Backend: `#[sqlx::test]` in `apps/api/tests/api.rs` — each test gets its own
  DB; build the router with `api::app::build` + `MemoryStorage`.
- The OpenAPI spec is snapshot-tested (insta): intentional API changes need
  `INSTA_UPDATE=always cargo test` (or `cargo insta review`) and a re-run of
  `just gen-api`.
- Frontend: Vitest, colocated `*.test.ts` under `apps/app/src`.
