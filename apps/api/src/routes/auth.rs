//! Register / login / logout / me. Cookie sessions; the session id is rotated
//! on login (session-fixation defence comes from `axum-login`).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{AuthSession, Credentials, CurrentUser, User, password};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::telemetry::Ctx;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(register))
        .routes(routes!(login))
        .routes(routes!(logout))
        .routes(routes!(me))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RoleDto {
    Admin,
    User,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: RoleDto,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            role: match user.role {
                crate::auth::Role::Admin => RoleDto::Admin,
                crate::auth::Role::User => RoleDto::User,
            },
            created_at: user.created_at,
        }
    }
}

// The `schema` attributes land in the OpenAPI spec, so the generated Zod
// client enforces the same rules in the browser as `validate` does here.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(format = Email)]
    pub email: String,
    #[validate(length(min = 8, max = 128, message = "must be 8 to 128 characters"))]
    #[schema(min_length = 8, max_length = 128)]
    pub password: String,
    #[validate(length(min = 1, max = 100, message = "must be 1 to 100 characters"))]
    #[schema(min_length = 1, max_length = 100)]
    pub display_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(format = Email)]
    pub email: String,
    #[schema(min_length = 1)]
    pub password: String,
}

/// Create an account and open a session.
#[utoipa::path(
    post,
    path = "/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created and logged in", body = UserDto),
        (status = 409, description = "Email already registered"),
        (status = 422, description = "Validation failed"),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    mut auth_session: AuthSession,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<UserDto>)> {
    body.validate()?;

    let password = body.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || password::hash(&password))
        .await
        .map_err(|e| AppError::Internal(e.into()))??;

    let id = Uuid::now_v7();
    let email = body.email.trim().to_lowercase();

    let result = sqlx::query!(
        r#"INSERT INTO users (id, email, password_hash, display_name)
           VALUES ($1, $2, $3, $4)"#,
        id,
        email,
        password_hash,
        body.display_name.trim()
    )
    .execute(&state.pool)
    .await;

    if let Err(err) = result {
        if err
            .as_database_error()
            .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
        {
            return Err(AppError::Conflict("email already registered".to_owned()));
        }
        return Err(err.into());
    }

    let user = axum_login::AuthnBackend::get_user(&auth_session.backend, &id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("user fetch failed: {e}")))?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("user vanished after insert")))?;

    auth_session
        .login(&user)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("session login failed: {e}")))?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// Log in with email + password.
#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in", body = UserDto),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login(
    ctx: Option<axum::Extension<Ctx>>,
    mut auth_session: AuthSession,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<UserDto>> {
    let creds = Credentials {
        email: body.email,
        password: body.password,
    };

    let user = auth_session
        .authenticate(creds)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("authentication failed: {e}")))?
        .ok_or(AppError::InvalidCredentials)?;

    auth_session
        .login(&user)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("session login failed: {e}")))?;

    if let Some(axum::Extension(ctx)) = ctx {
        ctx.set("user_id", user.id.to_string());
    }

    Ok(Json(user.into()))
}

/// End the current session.
#[utoipa::path(
    post,
    path = "/logout",
    tag = "auth",
    responses((status = 204, description = "Logged out")),
)]
pub async fn logout(mut auth_session: AuthSession) -> AppResult<StatusCode> {
    auth_session
        .logout()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("logout failed: {e}")))?;
    Ok(StatusCode::NO_CONTENT)
}

/// The currently authenticated user.
#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    responses(
        (status = 200, body = UserDto),
        (status = 401, description = "Not authenticated"),
    )
)]
pub async fn me(CurrentUser(user): CurrentUser) -> Json<UserDto> {
    Json(user.into())
}
