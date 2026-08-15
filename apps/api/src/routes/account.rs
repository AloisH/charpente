//! Account lifecycle: GDPR erasure and data export. Cheap to have from day
//! one, awkward to retrofit under a deadline.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::auth::{AuthSession, CurrentUser};
use crate::error::{AppError, AppResult};
use crate::routes::auth::UserDto;
use crate::routes::uploads::UploadDto;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(delete_account))
        .routes(routes!(export_account))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountExport {
    pub user: UserDto,
    pub uploads: Vec<UploadDto>,
}

/// Erase the account (GDPR art. 17). Deletes the user row; uploads cascade.
#[utoipa::path(
    delete,
    path = "",
    tag = "account",
    responses(
        (status = 204, description = "Account erased"),
        (status = 401, description = "Not authenticated"),
    )
)]
pub async fn delete_account(
    State(state): State<AppState>,
    mut auth_session: AuthSession,
    CurrentUser(user): CurrentUser,
) -> AppResult<StatusCode> {
    // Audit before the row disappears (actor becomes NULL via ON DELETE SET NULL).
    sqlx::query!(
        r#"INSERT INTO audit_log (id, actor_id, action, subject)
           VALUES ($1, $2, 'account.delete', $3)"#,
        Uuid::now_v7(),
        user.id,
        user.id.to_string()
    )
    .execute(&state.pool)
    .await?;

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&state.pool)
        .await?;

    auth_session
        .logout()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("logout failed: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Export the account's data as JSON (GDPR art. 20).
#[utoipa::path(
    get,
    path = "/export",
    tag = "account",
    responses(
        (status = 200, body = AccountExport),
        (status = 401, description = "Not authenticated"),
    )
)]
pub async fn export_account(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<AccountExport>> {
    let uploads = crate::routes::uploads::list_all_for_owner(&state.pool, user.id).await?;
    Ok(Json(AccountExport {
        user: user.into(),
        uploads,
    }))
}
