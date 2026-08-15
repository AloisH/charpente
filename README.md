# charpente

A fullstack starter: **Rust/Axum API + Vue 3 SPA + Nuxt vitrine**, with types
generated end-to-end from the backend's OpenAPI spec. Distributed as a
[Copier](https://copier.readthedocs.io) template so fixes made later propagate
to already-generated projects.

## Stack

| Layer   | Tech                                                                                       |
| ------- | ------------------------------------------------------------------------------------------ |
| API     | Rust, Axum 0.8, SQLx 0.8 (Postgres), tower-sessions + axum-login (cookie sessions), utoipa |
| SPA     | Vue 3.5, Vite, Tailwind v4, shadcn-vue, TanStack Query, vue-i18n (FR/EN)                   |
| Site    | Nuxt 4 SSR, @nuxtjs/i18n (FR/EN, hreflang, localized sitemap)                              |
| Types   | `openapi.json` (from Rust) → Hey API → TanStack Query helpers + Zod schemas                |
| Storage | Any S3 (RustFS in dev) via presigned direct-to-storage uploads                             |
| Logging | Wide events: one structured JSON line per request                                          |
| Deploy  | Two Docker images (GHCR) + docker compose + Caddy (auto-TLS)                               |

## Start a new project

```sh
pipx install copier
copier copy --trust gh:AloisH/charpente my-project
```

(`--trust` because the template runs one post-copy task:
`scripts/copier-rename.sh`, which renames everything to your project slug.)

Later, pull starter improvements into your project with `copier update`
(three-way merge; `.copier-answers.yml` records what you chose).

## Development

```sh
mise install              # pinned node / pnpm / rust / just
cp .env.example .env      # then set SESSION_KEY (openssl rand -base64 64)
pnpm install
just infra                # postgres + rustfs + adminer + mailpit (docker)
just migrate && just seed
just dev                  # api (watch) + codegen loop + vite + nuxt
```

- App: http://localhost:5173 (API proxied same-origin under `/api`)
- API docs (Scalar): http://localhost:8080/api/docs
- Site: http://localhost:3000 · Adminer: http://localhost:8081 · Mailpit: http://localhost:8025

Rename a field in a Rust DTO and the Vue component using it goes red a few
seconds later: `cargo watch` re-dumps `openapi.json` on every successful build,
a watcher re-runs Hey API into `packages/api-client/src/generated`, Vite HMR
picks it up. CI's `generated-check` job fails on any drift.

## Everyday commands

```sh
just lint       # rustfmt + clippy -D warnings + oxfmt + oxlint + vue-tsc
just test       # cargo tests (transactional DB per test) + vitest
just e2e        # playwright against the stack
just gen-api    # re-dump spec + regenerate client (also runs inside just dev)
just sqlx-prepare  # refresh committed .sqlx offline data after query changes
just migrate-new <name>
just release app patch   # tag v1.2.4 → CI builds + pushes the image
just release site minor  # tag site-v1.1.0
```

## Conventions (decided once, inherited everywhere)

- **API**: `/api/v1` prefix; RFC 9457 `problem+json` errors with a stable
  `code` (generated TS union — exhaustive `switch`); cursor pagination
  (`?cursor=&limit=` → `{ items, next_cursor, has_more }`); per-IP rate limit;
  1 MiB body cap (files never travel through the API — presigned PUT direct to
  storage).
- **DB**: UUIDv7 keys (time-ordered, index-friendly), `timestamptz` UTC,
  `updated_at` by trigger, soft delete only where justified.
- **Auth**: argon2id (explicit params), sessions in Postgres, signed HttpOnly
  SameSite=Lax cookie, session rotated at login, generic 401 on bad
  credentials (with timing-safe dummy verify). Roles `admin | user` behind a
  permission layer (`Role::permissions` in Rust ↔ `can()` / `<Can>` in Vue).
- **Account lifecycle**: GDPR erasure + JSON export endpoints, audit log for
  role-guarded actions.
- **Observability**: one wide event per request (`request_id` returned as
  `x-request-id`); the SPA posts failure events back with that id.

## Deployment

CI stops at "image is in GHCR" — no auto-deploy by design. On the server:

```sh
# once: put docker-compose.yml + docker/Caddyfile + a filled .env on the host
docker compose pull && docker compose up -d     # or: just deploy
```

`.env` needs: `GHCR_OWNER`, `SITE_DOMAIN`, `APP_DOMAIN`, `POSTGRES_USER`,
`POSTGRES_PASSWORD`, `SESSION_KEY`, `S3_*` (point at RustFS on the host or any
hosted S3).

> **Before your first real deployment**: nothing in this stack backs up
> Postgres. Add a `pg_dump` sidecar or hosted backups first. See
> `docs/adr/0004-out-of-scope.md`.

## Quality gates

Fast, on every PR: rustfmt, clippy (pedantic, `unwrap`/`panic` denied),
cargo-nextest with a per-test transactional DB, `.sqlx` freshness,
oxfmt/oxlint, vue-tsc, vitest, knip, `generated-check`, gitleaks, Playwright
(+ axe accessibility) against the production image.

Nightly, never blocking: cargo-deny, cargo-mutants, Schemathesis fuzzing of
every endpoint straight from the spec, trivy scan of published images.

## Documentation

- `docs/adr/` — why each load-bearing decision was made (and what was
  deliberately left out).
- `AGENTS.md` — commands + conventions for coding agents.
