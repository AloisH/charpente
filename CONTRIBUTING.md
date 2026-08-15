# Contributing

## Setup

```sh
mise install
cp .env.example .env   # set SESSION_KEY: openssl rand -base64 64
pnpm install           # also installs the lefthook git hooks
just infra && just migrate && just seed
just dev
```

## Before pushing

`just lint && just test`. The pre-commit hook runs oxfmt/oxlint on staged
files and checks `cargo fmt`; commit messages must be conventional commits
(commitlint).

## Changing the API surface

1. Edit the Rust handler/DTO (utoipa annotations included).
2. `just gen-api` — regenerates `openapi.json` + `packages/api-client`.
3. `INSTA_UPDATE=always cargo test` if the spec snapshot changed on purpose.
4. `just sqlx-prepare` if queries changed.
5. Commit everything together; CI verifies all of it.

## Releases

`just release app patch` / `just release site minor` — tags trigger the
GHCR build; deployment stays manual (`just deploy` on the server).
