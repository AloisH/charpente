//! Email-verification tokens: 32 random bytes, stored hashed (a DB leak must
//! not yield usable links), single-use, 24h TTL. Re-issuing invalidates any
//! previous token for the user.

use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::state::AppState;

pub fn hash_token(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

/// 32 random bytes, base64url — shared by every emailed-token flow.
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Replace any pending token for the user and return the new raw token
/// (only ever sent by email — never stored, never logged).
pub async fn issue(pool: &PgPool, user_id: Uuid) -> anyhow::Result<String> {
    let token = generate_token();

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM email_verification_tokens WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO email_verification_tokens (token_hash, user_id, expires_at)
           VALUES ($1, $2, now() + interval '24 hours')"#,
        hash_token(&token),
        user_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(token)
}

/// Issue a token and email the verification link.
pub async fn issue_and_send(
    state: &AppState,
    user_id: Uuid,
    email: &str,
    display_name: &str,
) -> anyhow::Result<()> {
    let token = issue(&state.pool, user_id).await?;
    let link = format!(
        "{}/verify-email?token={token}",
        state.config.mail_link_base()
    );
    let body = format!(
        "Hi {display_name},\n\n\
         Confirm your email address by opening this link:\n\n{link}\n\n\
         The link is valid for 24 hours. If you did not create this account,\n\
         you can ignore this email."
    );
    state
        .mailer
        .send(email, "Confirm your email address", &body)
        .await
}

/// Consume a token: returns the verified user's id, or `None` when the token
/// is unknown or expired. Single-use either way.
pub async fn consume(pool: &PgPool, token: &str) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query!(
        r#"DELETE FROM email_verification_tokens
           WHERE token_hash = $1
           RETURNING user_id, (expires_at > now()) AS "valid!""#,
        hash_token(token)
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };
    if !row.valid {
        return Ok(None);
    }

    sqlx::query!(
        "UPDATE users SET email_verified_at = now() WHERE id = $1 AND email_verified_at IS NULL",
        row.user_id
    )
    .execute(pool)
    .await?;

    Ok(Some(row.user_id))
}
