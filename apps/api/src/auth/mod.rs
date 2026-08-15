//! Authentication (cookie sessions via `axum-login`) and authorization
//! (a small permission layer over the `admin | user` roles, so adding roles
//! later means editing [`Role::permissions`], not rewriting route guards).

pub mod password;
pub mod reset;
pub mod verification;

use std::collections::HashSet;
use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::telemetry::Ctx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }

    fn permissions(self) -> &'static [Permission] {
        match self {
            Self::Admin => &[Permission::ManageUsers, Permission::ViewAuditLog],
            Self::User => &[],
        }
    }
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Self::Admin),
            "user" => Ok(Self::User),
            other => Err(anyhow::anyhow!("unknown role in database: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    ManageUsers,
    ViewAuditLog,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: Role,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Raw row as stored; converted to [`User`] so a bad `role` value surfaces as
/// an explicit error instead of a panic.
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    display_name: String,
    role: String,
    email_verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = anyhow::Error;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            display_name: row.display_name,
            role: row.role.parse()?,
            email_verified_at: row.email_verified_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // Sessions are invalidated when the password changes.
        self.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Debug, Clone)]
pub struct Backend {
    pool: PgPool,
}

impl Backend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn fetch_user_by_email(&self, email: &str) -> Result<Option<User>, BackendError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, email, password_hash, display_name, role,
                      email_verified_at AS "email_verified_at: DateTime<Utc>",
                      created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>"
               FROM users WHERE lower(email) = lower($1)"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(User::try_from).transpose().map_err(Into::into)
    }
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = BackendError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = self.fetch_user_by_email(&creds.email).await?;

        // Password verification is CPU-bound; keep it off the async runtime.
        // A dummy verify runs when the user does not exist so response timing
        // does not reveal which emails are registered.
        tokio::task::spawn_blocking(move || {
            if let Some(user) = user {
                password::verify(&user.password_hash, &creds.password).then_some(user)
            } else {
                password::verify_dummy(&creds.password);
                None
            }
        })
        .await
        .map_err(Into::into)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, email, password_hash, display_name, role,
                      email_verified_at AS "email_verified_at: DateTime<Utc>",
                      created_at AS "created_at: DateTime<Utc>",
                      updated_at AS "updated_at: DateTime<Utc>"
               FROM users WHERE id = $1"#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(User::try_from).transpose().map_err(Into::into)
    }
}

impl AuthzBackend for Backend {
    type Permission = Permission;

    async fn get_user_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        Ok(user.role.permissions().iter().copied().collect())
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;

/// Session key remembering the real admin while the session is logged in as
/// someone else. Presence of the key == impersonation in progress; it survives
/// the user switch because `login()` only cycles unauthenticated sessions.
pub const IMPERSONATOR_SESSION_KEY: &str = "impersonator_id";

/// Extractor: the authenticated user, or a `problem+json` 401.
/// Also stamps `user_id`/`role` onto the request's wide event.
pub struct CurrentUser(pub User);

impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let user = auth.user.ok_or(AppError::Unauthorized)?;
        if let Some(ctx) = parts.extensions.get::<Ctx>() {
            ctx.set("user_id", user.id.to_string());
            ctx.set("role", user.role.as_str());
        }
        Ok(Self(user))
    }
}

/// Extractor: an authenticated user holding `perm`, or 401/403.
pub struct RequirePermission<const PERM: u8>(pub User);

pub const PERM_MANAGE_USERS: u8 = 0;
pub const PERM_VIEW_AUDIT_LOG: u8 = 1;

const fn perm_from_const(perm: u8) -> Permission {
    match perm {
        PERM_MANAGE_USERS => Permission::ManageUsers,
        _ => Permission::ViewAuditLog,
    }
}

impl<S: Send + Sync, const PERM: u8> FromRequestParts<S> for RequirePermission<PERM> {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
        let required = perm_from_const(PERM);
        if user.role.permissions().contains(&required) {
            Ok(Self(user))
        } else {
            Err(AppError::Forbidden)
        }
    }
}
