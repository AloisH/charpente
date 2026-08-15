# ADR 0004 — Deliberately out of scope (decided, not forgotten)

**Status**: accepted

| Not included                  | Why it's fine                                                  | Cost to add later                                                                                              |
| ----------------------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Organizations / multi-tenancy | Single-user products                                           | **High** — `org_id` on every table/query/guard. Decide _before_ writing schema on any project that needs teams |
| Realtime (SSE/WebSocket)      | Query refetch-on-focus covers most cases                       | Low — additive route + composable                                                                              |
| Background job queue          | Nothing needs it yet                                           | Low–medium — Postgres-backed queue beside the pool                                                             |
| Error tracking / metrics SaaS | Wide events to stdout (ADR 0003)                               | Low — sink change                                                                                              |
| **Postgres backups**          | Not chosen                                                     | Low to add, **unbounded** to skip — add a `pg_dump` sidecar before the first real deployment                   |
| Site analytics                | Skipped — which is why no cookie banner exists                 | Low, but reintroduces the consent question                                                                     |
| Email provider (production)   | SMTP via `SMTP_URL` (Mailpit in dev); email verification ships | Low — point `SMTP_URL` at any provider's SMTP endpoint                                                         |
| Password reset flow           | Not needed yet                                                 | Low — mirror the email-verification pattern; `password_reset_tokens` already exists                            |
