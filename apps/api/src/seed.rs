//! `just seed` / `api seed` — create the first admin account from env vars.
//! Idempotent: an existing user with the seed email is promoted, not duplicated.

use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::password;
use crate::config::Config;

pub async fn seed_admin(pool: &PgPool, config: &Config) -> anyhow::Result<()> {
    let (Some(email), Some(seed_password)) = (
        config.seed_admin_email.clone(),
        config.seed_admin_password.clone(),
    ) else {
        anyhow::bail!("SEED_ADMIN_EMAIL and SEED_ADMIN_PASSWORD must be set to seed");
    };

    let email = email.trim().to_lowercase();
    let password_hash =
        tokio::task::spawn_blocking(move || password::hash(&seed_password)).await??;

    sqlx::query!(
        r#"INSERT INTO users (id, email, password_hash, display_name, role, email_verified_at)
           VALUES ($1, $2, $3, 'Admin', 'admin', now())
           ON CONFLICT ((lower(email)))
           DO UPDATE SET role = 'admin', email_verified_at = now()"#,
        Uuid::now_v7(),
        email,
        password_hash
    )
    .execute(pool)
    .await?;

    tracing::info!(target: "wide_event", operation = "seed_admin", email, "admin account seeded");
    Ok(())
}
