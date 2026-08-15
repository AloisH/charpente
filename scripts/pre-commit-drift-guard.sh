#!/bin/sh
# Ultra-fast tripwires for drift that CI only catches the slow way (a compile
# + a database away). Pure `git diff --cached` + grep — no cargo, no node.
#
# Heuristics, not proofs: they look at added/removed lines in staged api code.
# False positive (e.g. a query merely moved by a refactor)? Run the suggested
# command — if it produces no diff, bypass once with `git commit -n`.
set -eu

files=$(git diff --cached --name-only --diff-filter=ACMRD)

api_rs=$(printf '%s\n' "$files" | grep -E '^apps/api/.*\.rs$' || true)

fail=0

if [ -n "$api_rs" ]; then
    # shellcheck disable=SC2086 -- filenames come from git, newline-split is fine here
    # Added/removed lines only; the +++/--- file headers are stripped.
    diff_txt=$(git diff --cached -U0 -- $api_rs | grep -vE '^(\+\+\+|---)' | grep -E '^[+-]' || true)

    # sqlx macro or SQL string changed, but the offline cache wasn't refreshed:
    # CI's `cargo sqlx prepare --check` (and offline clippy) will fail.
    if printf '%s\n' "$diff_txt" | grep -qE 'sqlx::query|\bSELECT\b|\bINSERT INTO\b|\bDELETE FROM\b|\bRETURNING\b|\bUPDATE [a-z_]+ SET\b|\bFROM\b|\bWHERE\b|\bORDER BY\b|\bGROUP BY\b|\bON CONFLICT\b|\bVALUES\b' \
        && ! printf '%s\n' "$files" | grep -q '^\.sqlx/'; then
        echo "✗ sqlx queries changed but no .sqlx/ files are staged."
        echo "  run: just sqlx-prepare    then stage the .sqlx diff"
        fail=1
    fi

    # OpenAPI surface (utoipa/schema/validate annotations) changed, but neither
    # the spec dump nor the generated client was regenerated: generated-check
    # will fail on drift.
    if printf '%s\n' "$diff_txt" | grep -qE 'utoipa|ToSchema|IntoParams|#\[schema|#\[validate' \
        && ! printf '%s\n' "$files" | grep -qE '^(openapi\.json$|packages/api-client/src/generated/)'; then
        echo "✗ API surface changed but openapi.json / generated client are not staged."
        echo "  run: just gen-api    (spec snapshot: INSTA_UPDATE=always cargo test)"
        fail=1
    fi
fi

# A migration can silently change the shape of existing queries; the cache
# only needs refreshing when it does, so this one is a note, not a failure.
if printf '%s\n' "$files" | grep -q '^apps/api/migrations/' \
    && ! printf '%s\n' "$files" | grep -q '^\.sqlx/'; then
    echo "note: migrations staged without .sqlx changes — if an existing query's"
    echo "      shape changed, run: just sqlx-prepare"
fi

exit $fail
