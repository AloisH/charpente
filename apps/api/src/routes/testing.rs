//! Test-only endpoints, compiled in solely with `--features test-endpoints`
//! (used by the e2e suite). Never part of a release build.

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;

use crate::error::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/__test/reset", post(reset))
}

/// Wipe all data and reseed the admin account.
async fn reset(State(state): State<AppState>) -> AppResult<StatusCode> {
    sqlx::query!("TRUNCATE users, uploads, audit_log, password_reset_tokens, email_verification_tokens CASCADE")
        .execute(&state.pool)
        .await?;
    sqlx::query("TRUNCATE tower_sessions.session")
        .execute(&state.pool)
        .await
        .ok();
    crate::seed::seed_admin(&state.pool, &state.config).await?;
    Ok(StatusCode::NO_CONTENT)
}
