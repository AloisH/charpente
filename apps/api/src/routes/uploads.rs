//! Presigned-upload flow: the API validates and records the upload, the
//! browser PUTs the bytes straight to S3-compatible storage, then confirms.
//! Large bodies never pass through the Rust process.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;
use validator::Validate;

use crate::auth::CurrentUser;
use crate::error::{AppError, AppResult};
use crate::pagination::{CursorParams, Page};
use crate::state::AppState;

/// Server-side ceiling on declared upload size (50 MiB). The presigned URL is
/// content-type-bound; size is enforced at confirmation time.
const MAX_UPLOAD_BYTES: i64 = 50 * 1024 * 1024;

const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/gif",
    "application/pdf",
    "text/plain",
];

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create_upload))
        .routes(routes!(complete_upload))
        .routes(routes!(list_uploads))
        .routes(routes!(download_upload))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UploadState {
    Pending,
    Complete,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadDto {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub state: UploadState,
    pub created_at: DateTime<Utc>,
}

struct UploadRow {
    id: Uuid,
    filename: String,
    content_type: String,
    size_bytes: i64,
    state: String,
    created_at: DateTime<Utc>,
}

impl From<UploadRow> for UploadDto {
    fn from(row: UploadRow) -> Self {
        Self {
            id: row.id,
            filename: row.filename,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            state: if row.state == "complete" {
                UploadState::Complete
            } else {
                UploadState::Pending
            },
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUploadRequest {
    #[validate(length(min = 1, max = 255, message = "must be 1 to 255 characters"))]
    #[schema(min_length = 1, max_length = 255)]
    pub filename: String,
    pub content_type: String,
    #[validate(range(min = 1, message = "must be at least 1 byte"))]
    #[schema(minimum = 1)]
    pub size_bytes: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateUploadResponse {
    pub upload: UploadDto,
    /// Presigned URL: PUT the file bytes here with the declared content type.
    pub put_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DownloadUploadResponse {
    /// Presigned URL, valid for a few minutes.
    pub get_url: String,
}

/// Declare an upload: validates type + size, records a pending row, returns a
/// presigned PUT URL.
#[utoipa::path(
    post,
    path = "",
    tag = "uploads",
    request_body = CreateUploadRequest,
    responses(
        (status = 201, body = CreateUploadResponse),
        (status = 401, description = "Not authenticated"),
        (status = 422, description = "Validation failed"),
    )
)]
pub async fn create_upload(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<CreateUploadRequest>,
) -> AppResult<(StatusCode, Json<CreateUploadResponse>)> {
    body.validate()?;

    let mut errors = std::collections::BTreeMap::new();
    if !ALLOWED_CONTENT_TYPES.contains(&body.content_type.as_str()) {
        errors.insert(
            "content_type".to_owned(),
            vec![format!(
                "must be one of: {}",
                ALLOWED_CONTENT_TYPES.join(", ")
            )],
        );
    }
    if body.size_bytes > MAX_UPLOAD_BYTES {
        errors.insert(
            "size_bytes".to_owned(),
            vec![format!("must be at most {MAX_UPLOAD_BYTES} bytes")],
        );
    }
    if !errors.is_empty() {
        return Err(AppError::Validation(errors));
    }

    let id = Uuid::now_v7();
    let key = format!("uploads/{}/{id}", user.id);

    let row = sqlx::query_as!(
        UploadRow,
        r#"INSERT INTO uploads (id, owner_id, key, filename, content_type, size_bytes)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, filename, content_type, size_bytes, state, created_at AS "created_at: DateTime<Utc>""#,
        id,
        user.id,
        key,
        body.filename,
        body.content_type,
        body.size_bytes
    )
    .fetch_one(&state.pool)
    .await?;

    let put_url = state.storage.presign_put(&key, &body.content_type).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateUploadResponse {
            upload: row.into(),
            put_url,
        }),
    ))
}

/// Confirm the browser finished its PUT; verifies the object actually exists.
#[utoipa::path(
    post,
    path = "/{id}/complete",
    tag = "uploads",
    params(("id" = Uuid, Path, description = "Upload id")),
    responses(
        (status = 200, body = UploadDto),
        (status = 404, description = "No such pending upload"),
        (status = 409, description = "Object not found in storage"),
    )
)]
pub async fn complete_upload(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UploadDto>> {
    let key = sqlx::query_scalar!(
        "SELECT key FROM uploads WHERE id = $1 AND owner_id = $2",
        id,
        user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if !state.storage.exists(&key).await? {
        return Err(AppError::Conflict(
            "object has not been uploaded yet".to_owned(),
        ));
    }

    let row = sqlx::query_as!(
        UploadRow,
        r#"UPDATE uploads SET state = 'complete'
           WHERE id = $1 AND owner_id = $2
           RETURNING id, filename, content_type, size_bytes, state, created_at AS "created_at: DateTime<Utc>""#,
        id,
        user.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(row.into()))
}

/// The caller's uploads, newest first, cursor-paginated.
#[utoipa::path(
    get,
    path = "",
    tag = "uploads",
    params(CursorParams),
    responses(
        (status = 200, body = Page<UploadDto>),
        (status = 401, description = "Not authenticated"),
    )
)]
pub async fn list_uploads(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(params): Query<CursorParams>,
) -> AppResult<Json<Page<UploadDto>>> {
    let limit = params.limit();
    let rows = sqlx::query_as!(
        UploadRow,
        r#"SELECT id, filename, content_type, size_bytes, state, created_at AS "created_at: DateTime<Utc>"
           FROM uploads
           WHERE owner_id = $1 AND ($2::uuid IS NULL OR id < $2)
           ORDER BY id DESC
           LIMIT $3"#,
        user.id,
        params.cursor,
        limit + 1
    )
    .fetch_all(&state.pool)
    .await?;

    let items: Vec<UploadDto> = rows.into_iter().map(Into::into).collect();
    Ok(Json(Page::from_rows(items, limit, |u| u.id)))
}

/// A short-lived download URL for a completed upload.
#[utoipa::path(
    get,
    path = "/{id}/download",
    tag = "uploads",
    params(("id" = Uuid, Path, description = "Upload id")),
    responses(
        (status = 200, body = DownloadUploadResponse),
        (status = 404, description = "No such completed upload"),
    )
)]
pub async fn download_upload(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DownloadUploadResponse>> {
    let key = sqlx::query_scalar!(
        "SELECT key FROM uploads WHERE id = $1 AND owner_id = $2 AND state = 'complete'",
        id,
        user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let get_url = state.storage.presign_get(&key).await?;
    Ok(Json(DownloadUploadResponse { get_url }))
}

/// Everything a user owns — used by the account export.
pub async fn list_all_for_owner(pool: &sqlx::PgPool, owner_id: Uuid) -> AppResult<Vec<UploadDto>> {
    let rows = sqlx::query_as!(
        UploadRow,
        r#"SELECT id, filename, content_type, size_bytes, state, created_at AS "created_at: DateTime<Utc>"
           FROM uploads WHERE owner_id = $1 ORDER BY id DESC"#,
        owner_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
