//! Serving the built SPA. With `--features embed-spa` (prod image) the Vite
//! `dist/` is embedded in the binary — one artifact, no volume mounts. Without
//! it (dev), Vite serves the frontend and unknown routes 404.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

use crate::error::AppError;

/// Fallback for routes no handler matched. `/api/*` misses stay JSON; anything
/// else is the SPA's business.
pub async fn fallback(uri: Uri) -> Response {
    if uri.path().starts_with("/api/") {
        return AppError::NotFound.into_response();
    }
    spa_response(uri.path())
}

#[cfg(feature = "embed-spa")]
#[derive(rust_embed::RustEmbed)]
#[folder = "../app/dist"]
struct Assets;

#[cfg(feature = "embed-spa")]
fn spa_response(path: &str) -> Response {
    let trimmed = path.trim_start_matches('/');

    if let Some(asset) = Assets::get(trimmed) {
        let mime = mime_guess::from_path(trimmed).first_or_octet_stream();
        // Vite emits content-hashed filenames under assets/ — cache forever.
        let cache = if trimmed.starts_with("assets/") {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        return (
            [
                (header::CONTENT_TYPE, mime.as_ref().to_owned()),
                (header::CACHE_CONTROL, cache.to_owned()),
            ],
            asset.data.into_owned(),
        )
            .into_response();
    }

    // Client-side routes get index.html; the SPA router takes it from there.
    match Assets::get("index.html") {
        Some(index) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8".to_owned()),
                (header::CACHE_CONTROL, "no-cache".to_owned()),
            ],
            index.data.into_owned(),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "SPA assets missing from binary").into_response(),
    }
}

#[cfg(not(feature = "embed-spa"))]
fn spa_response(_path: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        "SPA not embedded in this build — run the Vite dev server (just dev)",
    )
        .into_response()
}
