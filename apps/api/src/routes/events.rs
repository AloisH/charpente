//! Client-side wide events. A failed mutation in the SPA posts one event here
//! (with the `request_id` it got back from the API), so frontend failures join
//! up with backend events in the same log stream.

use axum::Json;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(ingest_event))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClientEvent {
    /// Event name, e.g. "mutation_failed".
    pub name: String,
    /// Request id returned by the failing API call, if any.
    pub request_id: Option<String>,
    /// Arbitrary structured context.
    pub fields: Option<serde_json::Value>,
}

/// Ingest one client-side wide event (fire-and-forget; rate-limited).
#[utoipa::path(
    post,
    path = "",
    tag = "events",
    request_body = ClientEvent,
    responses((status = 202, description = "Accepted")),
)]
pub async fn ingest_event(Json(event): Json<ClientEvent>) -> StatusCode {
    let fields = event.fields.map(|f| f.to_string()).unwrap_or_default();
    tracing::info!(
        target: "client_event",
        name = event.name,
        request_id = event.request_id,
        fields,
        "client_event"
    );
    StatusCode::ACCEPTED
}
