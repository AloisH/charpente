//! Role-guarded routes. The guard is the `RequirePermission` extractor, so
//! adding roles later means touching `Role::permissions`, not every handler.
//! Role changes land in `audit_log` — the durable "who changed what".

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::auth::{PERM_MANAGE_USERS, PERM_VIEW_AUDIT_LOG, RequirePermission};
use crate::error::{AppError, AppResult};
use crate::pagination::{CursorParams, Page};
use crate::routes::auth::{RoleDto, UserDto};
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_users))
        .routes(routes!(set_user_role))
        .routes(routes!(list_audit_log))
}

struct UserListRow {
    id: Uuid,
    email: String,
    display_name: String,
    role: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<UserListRow> for UserDto {
    type Error = AppError;

    fn try_from(row: UserListRow) -> Result<Self, Self::Error> {
        let role = match row.role.as_str() {
            "admin" => RoleDto::Admin,
            "user" => RoleDto::User,
            other => {
                return Err(AppError::Internal(anyhow::anyhow!("unknown role: {other}")));
            }
        };
        Ok(Self {
            id: row.id,
            email: row.email,
            display_name: row.display_name,
            role,
            created_at: row.created_at,
        })
    }
}

/// List all users (admin only), newest first, cursor-paginated.
#[utoipa::path(
    get,
    path = "/users",
    tag = "admin",
    params(CursorParams),
    responses(
        (status = 200, body = Page<UserDto>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not an admin"),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    RequirePermission(_admin): RequirePermission<PERM_MANAGE_USERS>,
    Query(params): Query<CursorParams>,
) -> AppResult<Json<Page<UserDto>>> {
    let limit = params.limit();
    let rows = sqlx::query_as!(
        UserListRow,
        r#"SELECT id, email, display_name, role, created_at AS "created_at: DateTime<Utc>"
           FROM users
           WHERE ($1::uuid IS NULL OR id < $1)
           ORDER BY id DESC
           LIMIT $2"#,
        params.cursor,
        limit + 1
    )
    .fetch_all(&state.pool)
    .await?;

    let users = rows
        .into_iter()
        .map(UserDto::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(Page::from_rows(users, limit, |u| u.id)))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetRoleRequest {
    pub role: RoleDto,
}

/// Change a user's role (admin only). Recorded in the audit log.
#[utoipa::path(
    patch,
    path = "/users/{id}/role",
    tag = "admin",
    request_body = SetRoleRequest,
    params(("id" = Uuid, Path, description = "User id")),
    responses(
        (status = 200, body = UserDto),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "No such user"),
    )
)]
pub async fn set_user_role(
    State(state): State<AppState>,
    RequirePermission(admin): RequirePermission<PERM_MANAGE_USERS>,
    Path(id): Path<Uuid>,
    Json(body): Json<SetRoleRequest>,
) -> AppResult<Json<UserDto>> {
    let role = match body.role {
        RoleDto::Admin => "admin",
        RoleDto::User => "user",
    };

    let row = sqlx::query_as!(
        UserListRow,
        r#"UPDATE users SET role = $2 WHERE id = $1
           RETURNING id, email, display_name, role, created_at AS "created_at: DateTime<Utc>""#,
        id,
        role
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query!(
        r#"INSERT INTO audit_log (id, actor_id, action, subject, detail)
           VALUES ($1, $2, 'user.set_role', $3, $4)"#,
        Uuid::now_v7(),
        admin.id,
        id.to_string(),
        serde_json::json!({ "role": role })
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(row.try_into()?))
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct AuditLogEntryDto {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub subject: Option<String>,
    pub detail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// The audit trail of role-guarded actions, newest first.
#[utoipa::path(
    get,
    path = "/audit-log",
    tag = "admin",
    params(CursorParams),
    responses(
        (status = 200, body = Page<AuditLogEntryDto>),
        (status = 403, description = "Missing permission"),
    )
)]
pub async fn list_audit_log(
    State(state): State<AppState>,
    RequirePermission(_admin): RequirePermission<PERM_VIEW_AUDIT_LOG>,
    Query(params): Query<CursorParams>,
) -> AppResult<Json<Page<AuditLogEntryDto>>> {
    let limit = params.limit();
    let rows = sqlx::query_as!(
        AuditLogEntryDto,
        r#"SELECT id, actor_id, action, subject,
                  detail AS "detail: serde_json::Value",
                  created_at AS "created_at: DateTime<Utc>"
           FROM audit_log
           WHERE ($1::uuid IS NULL OR id < $1)
           ORDER BY id DESC
           LIMIT $2"#,
        params.cursor,
        limit + 1
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(Page::from_rows(rows, limit, |e| e.id)))
}
