# charpente — single entry point for every command.
# `just` with no args lists the recipes.

set dotenv-load := true

default:
    @just --list

# One-time machine setup: cargo tools the dev loop needs
setup:
    cargo install cargo-watch
    cargo install sqlx-cli --no-default-features --features rustls,postgres
    pnpm install

# ── Dev ──────────────────────────────────────────────────────────

# Start local infra (postgres, rustfs, adminer, mailpit)
# rustfs-init is a one-shot (creates the bucket, exits 0), which `--wait`
# would count as a failure — so it runs separately.
infra:
    docker compose -f docker-compose.dev.yml up -d --wait postgres rustfs adminer mailpit
    docker compose -f docker-compose.dev.yml run --rm rustfs-init

infra-down:
    docker compose -f docker-compose.dev.yml down

# Full dev loop: infra + api (watch, re-emits openapi.json) + client codegen watcher + vite + nuxt
dev: infra
    pnpm exec concurrently -k -n api,gen,app,site -c blue,magenta,green,yellow \
        "just dev-api" \
        "node scripts/watch-openapi.mjs" \
        "pnpm --filter @charpente/app dev" \
        "pnpm --filter @charpente/site dev"

# API only, rebuild + restart on change; each successful build re-dumps openapi.json
dev-api:
    cargo watch -w apps/api/src -w apps/api/migrations -x "run -p api -- dump-openapi" -x "run -p api"

# ── Codegen ──────────────────────────────────────────────────────

# Dump openapi.json from the Rust source of truth
gen-openapi:
    cargo run -p api -- dump-openapi

# Regenerate the typed client from openapi.json
gen-client:
    pnpm exec openapi-ts

# Full pipeline: spec + client
gen-api: gen-openapi gen-client

# ── Database ─────────────────────────────────────────────────────

migrate:
    cargo sqlx migrate run --source apps/api/migrations

migrate-new name:
    cargo sqlx migrate add --source apps/api/migrations {{name}}

# Refresh committed offline query data (.sqlx/) — run after changing queries
sqlx-prepare:
    cargo sqlx prepare --workspace -- --all-targets --all-features

seed:
    cargo run -p api -- seed

# ── Quality ──────────────────────────────────────────────────────

fmt:
    cargo fmt --all
    pnpm exec oxfmt .

lint:
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings
    pnpm exec oxfmt --check .
    pnpm exec oxlint --deny-warnings
    pnpm --filter @charpente/app typecheck

test:
    cargo test --workspace
    pnpm --filter @charpente/app test

e2e:
    pnpm --filter e2e test

# ── Release ──────────────────────────────────────────────────────

# just release app patch | just release site minor
release target bump:
    ./scripts/release.sh {{target}} {{bump}}

# Pull + restart the prod stack on the current host (run on the server)
deploy:
    docker compose pull && docker compose up -d
