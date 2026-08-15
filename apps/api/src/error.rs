//! One error type for the whole API, rendered as RFC 9457 `problem+json`
//! with a stable machine-readable `code` the generated TS client can
//! exhaustively `switch` on.

use std::collections::BTreeMap;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// Stable machine-readable error codes. Additions are fine; renames are a
/// breaking API change surfaced by the OpenAPI diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ValidationFailed,
    InvalidCredentials,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    PayloadTooLarge,
    RateLimited,
    Internal,
}

/// RFC 9457 problem details, extended with `code` and field-level `errors`.
#[derive(Debug, Serialize, ToSchema)]
pub struct Problem {
    /// Always "about:blank"; the machine-readable discriminant is `code`.
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Field-level validation messages, keyed by field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation failed")]
    Validation(BTreeMap<String, Vec<String>>),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("authentication required")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Conflict(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            other => Self::Internal(other.into()),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errs: validator::ValidationErrors) -> Self {
        let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (field, errors) in errs.field_errors() {
            let messages = errors
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map_or_else(|| e.code.to_string(), ToString::to_string)
                })
                .collect();
            fields.insert(field.to_string(), messages);
        }
        Self::Validation(fields)
    }
}

/// Carried in response extensions so the wide-event middleware can attach
/// error details to the request's single log line.
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub kind: &'static str,
    pub message: String,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InvalidCredentials | Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> ErrorCode {
        match self {
            Self::Validation(_) => ErrorCode::ValidationFailed,
            Self::InvalidCredentials => ErrorCode::InvalidCredentials,
            Self::Unauthorized => ErrorCode::Unauthorized,
            Self::Forbidden => ErrorCode::Forbidden,
            Self::NotFound => ErrorCode::NotFound,
            Self::Conflict(_) => ErrorCode::Conflict,
            Self::Internal(_) => ErrorCode::Internal,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::InvalidCredentials => "invalid_credentials",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Conflict(_) => "conflict",
            Self::Internal(_) => "internal",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();
        let info = ErrorInfo {
            kind: self.kind(),
            message: match &self {
                // Never leak internal error chains to clients.
                Self::Internal(e) => format!("{e:#}"),
                other => other.to_string(),
            },
        };

        let (detail, errors) = match self {
            Self::Validation(fields) => (None, Some(fields)),
            Self::Internal(_) => (Some("internal server error".to_owned()), None),
            other => (Some(other.to_string()), None),
        };

        let problem = Problem {
            problem_type: "about:blank".to_owned(),
            title: status.canonical_reason().unwrap_or("Error").to_owned(),
            status: status.as_u16(),
            code,
            detail,
            errors,
        };

        let mut response = (status, Json(problem)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response.extensions_mut().insert(info);
        response
    }
}

pub type AppResult<T> = Result<T, AppError>;
