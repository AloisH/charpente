//! Router assembly: sessions, auth, rate limiting, security headers, wide
//! events. Built once here so integration tests exercise the exact production
//! router.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::Router;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware;
use axum_login::AuthManagerLayerBuilder;
use base64::Engine;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_sessions::cookie::{Key, SameSite};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use utoipa_axum::router::OpenApiRouter;

use crate::auth::Backend;
use crate::state::AppState;
use crate::{openapi, routes, static_files, telemetry};

/// Global request-body ceiling. File bytes never hit the API (presigned
/// uploads go straight to storage), so 1 MiB of JSON is plenty.
const BODY_LIMIT_BYTES: usize = 1024 * 1024;

pub async fn build(state: AppState) -> anyhow::Result<Router> {
    let config = Arc::clone(&state.config);

    // Sessions in Postgres, signed cookies.
    let session_store = PostgresStore::new(state.pool.clone());
    session_store
        .migrate()
        .await
        .context("session store migration failed")?;

    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(config.session_key.trim())
        .context("SESSION_KEY must be valid base64")?;
    let key = Key::try_from(key_bytes.as_slice())
        .context("SESSION_KEY must decode to at least 64 bytes")?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("charpente_session")
        .with_secure(!config.app_env.is_dev())
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(14)))
        .with_signed(key);

    let auth_layer =
        AuthManagerLayerBuilder::new(Backend::new(state.pool.clone()), session_layer).build();

    // Per-IP rate limit on /api only — static assets and health checks must
    // never eat the budget.
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(120)
            .finish()
            .context("invalid rate-limit configuration")?,
    );

    let (api_router, api_doc) = OpenApiRouter::with_openapi(openapi::document())
        .nest("/api/v1", routes::api_v1())
        .split_for_parts();
    let api_router = api_router.layer(GovernorLayer::new(governor_config));

    let mut router = api_router.merge(routes::health::router());

    #[cfg(feature = "test-endpoints")]
    {
        router = router.merge(routes::testing::router());
    }

    // Interactive API docs, dev only.
    if config.app_env.is_dev() {
        use utoipa_scalar::{Scalar, Servable};
        router = router.merge(Scalar::with_url("/api/docs", api_doc));
    }

    let mut router = router
        .fallback(static_files::fallback)
        .layer(auth_layer)
        .layer(middleware::from_fn(telemetry::wide_event))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(RequestBodyLimitLayer::new(BODY_LIMIT_BYTES))
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            // MIME sniffing is never wanted.
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            // Leak nothing cross-origin.
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            // Hashed-asset SPA: everything same-origin except direct-to-storage
            // uploads (connect-src) and data/blob images for previews.
            csp_header(config.s3_browser_endpoint())?,
        ));

    // In dev the SPA usually reaches the API through the Vite proxy
    // (same-origin); CORS only matters if APP_PUBLIC_URL points elsewhere.
    if let (true, Some(origin)) = (config.app_env.is_dev(), &config.app_public_url) {
        let cors = CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<HeaderValue>()
                    .context("APP_PUBLIC_URL is not a valid origin")?,
            )
            .allow_credentials(true)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::PUT,
                Method::DELETE,
            ])
            .allow_headers([header::CONTENT_TYPE]);
        router = router.layer(cors);
    }

    Ok(router.with_state(state))
}

fn csp_header(s3_endpoint: &str) -> anyhow::Result<HeaderValue> {
    let value = format!(
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: blob:; font-src 'self'; connect-src 'self' {s3_endpoint}; \
         frame-ancestors 'none'; base-uri 'self'"
    );
    HeaderValue::from_str(&value).context("invalid CSP header")
}
