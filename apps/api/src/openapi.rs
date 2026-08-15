//! The OpenAPI document, derived from the code (utoipa). `api dump-openapi`
//! writes it to `openapi.json`, from which the typed frontend client is
//! generated — the two sides cannot drift.

use utoipa::OpenApi;

use crate::routes;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "charpente API",
        description = "Backend API. Errors are RFC 9457 problem+json with a stable `code`.",
        version = env!("CARGO_PKG_VERSION"),
    ),
    components(schemas(crate::error::Problem, crate::error::ErrorCode)),
    tags(
        (name = "auth", description = "Registration and cookie-session authentication"),
        (name = "account", description = "Account lifecycle: erasure and export"),
        (name = "admin", description = "Admin-only user management"),
        (name = "uploads", description = "Presigned direct-to-storage uploads"),
        (name = "events", description = "Client-side wide-event ingestion"),
    )
)]
struct ApiDoc;

/// The complete spec for all `/api/v1` routes.
pub fn document() -> utoipa::openapi::OpenApi {
    let (_, api) = utoipa_axum::router::OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", routes::api_v1())
        .split_for_parts();
    api
}
