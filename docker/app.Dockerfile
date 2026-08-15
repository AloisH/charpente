# Build context: repository root.
#   docker build -f docker/app.Dockerfile .
# Stage 1 builds the SPA, stage 2/3 build the Rust binary with cargo-chef
# (dependency layer cached separately — the difference between 2-minute and
# 12-minute rebuilds), stage 4 is the runtime.

# ── 1. SPA ───────────────────────────────────────────────────────
FROM node:22-slim AS spa
RUN corepack enable
WORKDIR /repo
COPY pnpm-lock.yaml pnpm-workspace.yaml package.json tsconfig.base.json ./
COPY apps/app/package.json apps/app/
COPY apps/site/package.json apps/site/
COPY packages/api-client/package.json packages/api-client/
COPY packages/tokens/package.json packages/tokens/
COPY e2e/package.json e2e/
RUN pnpm install --frozen-lockfile --filter @charpente/app... --ignore-scripts
COPY packages/ packages/
COPY apps/app/ apps/app/
RUN pnpm --filter @charpente/app exec vite build

# ── 2. Rust dependency plan ─────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1.97-slim AS chef
WORKDIR /repo

FROM chef AS planner
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY apps/api/ apps/api/
RUN cargo chef prepare --recipe-path recipe.json

# ── 3. Rust build ───────────────────────────────────────────────
FROM chef AS builder
# embed-spa for releases; e2e adds test-endpoints via build-arg.
ARG CARGO_FEATURES=embed-spa
COPY --from=planner /repo/recipe.json recipe.json
RUN cargo chef cook --release --features "$CARGO_FEATURES" --recipe-path recipe.json
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .sqlx/ .sqlx/
COPY apps/api/ apps/api/
# The SPA is embedded into the binary (rust-embed).
COPY --from=spa /repo/apps/app/dist/ apps/app/dist/
ENV SQLX_OFFLINE=1
RUN cargo build --release -p api --features "$CARGO_FEATURES"

# ── 4. Runtime ──────────────────────────────────────────────────
FROM debian:trixie-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 app
COPY --from=builder /repo/target/release/api /usr/local/bin/api
USER app
ENV APP_HOST=0.0.0.0 APP_PORT=8080 APP_ENV=prod
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --start-period=10s \
    CMD curl -sf http://localhost:8080/health/ready || exit 1
ENTRYPOINT ["/usr/local/bin/api"]
