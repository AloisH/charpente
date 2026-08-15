# ADR 0002 — Cookie sessions, argon2id, permission indirection

**Status**: accepted

## Decision

Server-side sessions in Postgres (`tower-sessions` + `axum-login`), signed
HttpOnly SameSite=Lax cookie, rotated at login, rolling 14-day expiry.
Passwords are argon2id with explicit parameters (19 MiB, t=2, p=1). Roles are
`admin | user`, but route guards check _permissions_
(`RequirePermission<PERM_…>` extractors), mapped from roles in one place
(`Role::permissions`), mirrored in the SPA as `can()` / `<Can>`.

## Why

- Same-origin deployment (SPA embedded in the API binary, `/api` under the app
  domain) makes cookies the simplest correct choice — no token storage in JS,
  no CSRF token needed with SameSite=Lax (double-submit is documented for the
  day a cross-origin client appears).
- The permission indirection means adding a role edits one function, not every
  guard.
- Login runs a dummy argon2 verification when the account doesn't exist, so
  timing doesn't reveal registered emails; the error is a generic 401.

## Notes

Sessions are invalidated on password change via `session_auth_hash`.
Email uniqueness is a functional index on `lower(email)` (instead of citext:
same behavior, no extension, no sqlx type friction).
