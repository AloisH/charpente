# Build context: repository root.
#   docker build -f docker/site.Dockerfile .

FROM node:22-slim AS build
RUN corepack enable
WORKDIR /repo
COPY pnpm-lock.yaml pnpm-workspace.yaml package.json tsconfig.base.json ./
COPY apps/app/package.json apps/app/
COPY apps/site/package.json apps/site/
COPY packages/api-client/package.json packages/api-client/
COPY packages/tokens/package.json packages/tokens/
COPY e2e/package.json e2e/
RUN pnpm install --frozen-lockfile --filter @charpente/site... --ignore-scripts
COPY packages/tokens/ packages/tokens/
COPY apps/site/ apps/site/
RUN pnpm --filter @charpente/site exec nuxt build

FROM node:22-slim AS runtime
RUN useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /repo/apps/site/.output/ .output/
USER app
ENV NITRO_PORT=3000 NITRO_HOST=0.0.0.0 NODE_ENV=production
EXPOSE 3000
HEALTHCHECK --interval=15s --timeout=3s --start-period=10s \
    CMD node -e "fetch('http://localhost:3000/').then(r=>process.exit(r.ok?0:1)).catch(()=>process.exit(1))"
CMD ["node", ".output/server/index.mjs"]
