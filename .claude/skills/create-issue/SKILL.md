---
name: create-issue
description: Turn a rough bug report or feature idea into a well-grounded GitHub issue by first analyzing the codebase, then filing it with the gh CLI. Use when the user wants to create, file, or open a GitHub issue, or describes a bug/feature and says "make an issue for this".
---

# Create a grounded GitHub issue

Turn the user's rough description into an issue that a future agent (or human)
can pick up cold — grounded in real file paths, repo conventions, and
acceptance criteria. Never file an issue you haven't grounded in the code.

## Workflow

1. **Understand the request.** If the description is one vague sentence,
   ask at most one clarifying question; otherwise proceed.

2. **Analyze the codebase before writing anything.** Locate the code the
   issue touches (Grep/Glob/Read). Collect:
   - Concrete `path:line` references to the relevant modules.
   - Existing patterns the fix/feature should follow (nearest similar
     handler, component, test).
   - Repo chores the change will trigger — check `AGENTS.md`; in this repo
     that means: `just gen-api` for DTO/handler changes, `just sqlx-prepare`
     for query changes, `fr`+`en` i18n keys for user-facing strings, insta
     snapshot update for spec changes.

3. **Draft the issue body** (markdown):
   - **Problem / Motivation** — what and why, 2-4 sentences.
   - **Where in the code** — the `path:line` references with one line each
     on their role.
   - **Suggested approach** — short, concrete, referencing the existing
     pattern to imitate. Mark it as a suggestion, not a spec.
   - **Acceptance criteria** — checkboxes, testable.
   - **Chores checklist** — the triggered repo chores from step 2.

4. **File it.** Show the user the title, then:

   ```sh
   gh issue create --title "<type>: <imperative summary>" --body-file /tmp/issue-body.md
   ```

   Write the body to a temp file first (heredocs mangle backticks). Use the
   conventional-commit type vocabulary in the title (`fix:`, `feat:`, …).
   Add `--label` only for labels that exist (`gh label list`).

5. **Report the issue URL** back to the user.

## Guardrails

- One issue per problem — if the request bundles several, propose the split
  and file them separately.
- If while analyzing you find the "bug" is actually intended behavior or
  already fixed, say so instead of filing.
- Don't paste large code blocks into the issue; `path:line` references stay
  accurate longer and read better.
