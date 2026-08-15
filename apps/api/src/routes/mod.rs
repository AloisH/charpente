pub mod account;
pub mod admin;
pub mod auth;
pub mod events;
pub mod health;
#[cfg(feature = "test-endpoints")]
pub mod testing;
pub mod uploads;

use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

/// Every documented route. Routes and their OpenAPI docs register together —
/// a handler that is not documented is not routed.
pub fn api_v1() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .nest("/auth", auth::router())
        .nest("/account", account::router())
        .nest("/admin", admin::router())
        .nest("/uploads", uploads::router())
        .nest("/events", events::router())
}
