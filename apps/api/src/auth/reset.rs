//! Password-reset tokens: same shape as [`super::verification`] (random 32
//! bytes, stored hashed, single-use, rotated on re-issue) but a tighter
//! 1-hour TTL — reset links are a takeover vector, verification links aren't.

use sqlx::PgPool;
use uuid::Uuid;

use super::verification::{generate_token, hash_token};
use crate::state::AppState;

/// Look up the account and, when it exists, email a reset link. Silently does
/// nothing for unknown emails — the caller must answer identically either way
/// so responses never reveal whether an account exists.
pub async fn request(state: &AppState, email: &str) -> anyhow::Result<()> {
    let user = sqlx::query!(
        "SELECT id, display_name FROM users WHERE lower(email) = lower($1)",
        email
    )
    .fetch_optional(&state.pool)
    .await?;
    let Some(user) = user else { return Ok(()) };

    let token = issue(&state.pool, user.id).await?;
    let link = format!(
        "{}/reset-password?token={token}",
        state.config.mail_link_base()
    );
    let body = format!(
        "Hi {},\n\n\
         Someone asked to reset the password for this account. If that was\n\
         you, open this link (valid for 1 hour):\n\n{link}\n\n\
         If it wasn't you, ignore this email — your password is unchanged.",
        user.display_name
    );
    state.mailer.send(email, "Reset your password", &body).await
}

async fn issue(pool: &PgPool, user_id: Uuid) -> anyhow::Result<String> {
    let token = generate_token();

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM password_reset_tokens WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO password_reset_tokens (token_hash, user_id, expires_at)
           VALUES ($1, $2, now() + interval '1 hour')"#,
        hash_token(&token),
        user_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(token)
}

/// Consume a token: returns the user id, or `None` when unknown or expired.
/// Single-use either way.
pub async fn consume(pool: &PgPool, token: &str) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query!(
        r#"DELETE FROM password_reset_tokens
           WHERE token_hash = $1
           RETURNING user_id, (expires_at > now()) AS "valid!""#,
        hash_token(token)
    )
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some(row) if row.valid => Some(row.user_id),
        _ => None,
    })
}
