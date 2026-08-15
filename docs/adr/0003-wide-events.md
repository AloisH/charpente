# ADR 0003 — Wide events instead of scattered log lines

**Status**: accepted

## Decision

One structured event per request (loggingsucks.com's "canonical log line").
Middleware opens a per-request context (`telemetry::Ctx`); any handler or
extractor attaches fields (`ctx.set("user_id", …)`); exactly one JSON line is
emitted when the response leaves, with errors folded into the same event.
`request_id` is generated (or taken from `x-request-id`) and returned in the
response header. The SPA posts its own failure events to `/api/v1/events`
carrying that id.

## Why

One grep yields a failing request with its full context. The shape is exactly
what hosted tools (Sentry, Loki, ClickHouse…) ingest, so adding one later is a
sink change, not a rewrite — which is why no error-tracking SaaS ships in the
template.

## Notes

The `audit_log` table is deliberately separate: wide events answer "what
happened on this request", the audit log answers the auditor's "who changed
what, durably".
