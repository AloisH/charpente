---
name: work-issue
description: Pick up a GitHub issue (given its URL or number), implement it end to end, and close it — fetching context with the gh CLI, following the repo's hard rules, committing directly to main. Use when the user pastes a GitHub issue URL/number and asks to work on, implement, fix, or resolve it.
---

# Work a GitHub issue end to end

Input: an issue URL like `https://github.com/<owner>/<repo>/issues/42`, or
just `#42` for the current repo.

## Workflow

1. **Fetch the full issue** — body AND discussion; later comments often
   amend the spec:

   ```sh
   gh issue view <number> --json title,body,labels,comments,url
   ```

2. **Ground it.** Read the files the issue references and re-verify them —
   line numbers drift; the code may have moved since filing. Read `AGENTS.md`
   for the hard rules. If the issue is already fixed or its premise no longer
   holds, comment that on the issue and stop — don't implement a stale spec.

3. **Implement** following repo conventions. In this repo, run the matching
   chore right after the change that triggers it:
   - Rust DTO/handler changed → `just gen-api` (+ `INSTA_UPDATE=always cargo test`
     for the spec snapshot).
   - sqlx query added/changed → `just sqlx-prepare`, commit the `.sqlx` diff.
   - User-facing SPA/site string → add both `fr` and `en` i18n keys.
   - Never edit `packages/api-client/src/generated/` or `openapi.json` by hand.

4. **Add or update tests** proving the acceptance criteria. Backend:
   `#[sqlx::test]` in `apps/api/tests/api.rs`; frontend: colocated Vitest.

5. **Verify before committing** — both must pass locally:

   ```sh
   just lint
   just test
   ```

6. **Commit and push directly to `main`** (no PRs, no feature branches —
   house rule). Conventional commit; auto-close the issue from the body:

   ```sh
   git commit -m "<type>: <summary> (#<number>)" -m "Closes #<number>."
   git push
   ```

7. **Wrap up on the issue.** If anything deviated from the issue's plan, or
   acceptance criteria needed interpretation, leave a short comment:

   ```sh
   gh issue comment <number> --body "..."
   ```

   The `Closes #` footer closes the issue when the push lands; verify with
   `gh issue view <number> --json state`, and `gh issue close` only if
   auto-close didn't fire.

## Guardrails

- Stay inside the issue's scope; file follow-up issues for adjacent problems
  you notice instead of fixing them silently.
- If acceptance criteria are ambiguous and both readings are plausible, ask
  the user before implementing — not after.
- CI runs on push: check `gh run list --branch main --limit 1` afterwards and
  fix red CI before calling the issue done.
