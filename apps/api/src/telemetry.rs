//! Wide-event logging: one structured event per request, carrying everything
//! about that request, instead of scattered log lines to correlate later.
//!
//! Handlers, extractors and services attach fields via [`Ctx::set`]; the
//! middleware emits a single event when the response leaves. Errors join the
//! same event (via [`crate::error::ErrorInfo`] in response extensions), so one
//! grep yields the failing request with its full context.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::{ConnectInfo, MatchedPath, Request};
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::config::AppEnv;
use crate::error::ErrorInfo;

pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Per-request accumulator. Cloning shares the same underlying map.
#[derive(Debug, Clone, Default)]
pub struct Ctx(Arc<Mutex<Map<String, Value>>>);

impl Ctx {
    /// Attach a field to this request's wide event. Poisoned-lock failures are
    /// swallowed: telemetry must never take down a request.
    pub fn set(&self, key: &str, value: impl Into<Value>) {
        if let Ok(mut map) = self.0.lock() {
            map.insert(key.to_owned(), value.into());
        }
    }

    fn drain(&self) -> Map<String, Value> {
        self.0
            .lock()
            .map(|mut m| std::mem::take(&mut *m))
            .unwrap_or_default()
    }
}

/// Initialize tracing: pretty output for humans in dev, one JSON object per
/// line on stdout in prod (ready for `docker logs` or any collector).
pub fn init(env: AppEnv) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=warn,sqlx=warn"));

    if env.is_dev() {
        fmt().with_env_filter(filter).init();
    } else {
        fmt()
            .json()
            .flatten_event(true)
            .with_env_filter(filter)
            .init();
    }
}

/// The wide-event middleware. Applied once, outermost, on every route.
pub async fn wide_event(mut req: Request, next: Next) -> Response {
    let started = Instant::now();

    let request_id = req
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| Uuid::now_v7().to_string(), ToOwned::to_owned);

    let method = req.method().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| req.uri().path().to_owned(), |p| p.as_str().to_owned());
    let user_agent = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);
    let ip = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip().to_string());

    let ctx = Ctx::default();
    req.extensions_mut().insert(ctx.clone());

    let mut response = next.run(req).await;

    let status = response.status().as_u16();
    #[allow(clippy::cast_possible_truncation)]
    let duration_ms = started.elapsed().as_millis() as u64;
    let error = response.extensions().get::<ErrorInfo>().cloned();
    let extra = Value::Object(ctx.drain()).to_string();

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(REQUEST_ID_HEADER.clone(), value);
    }

    match error {
        Some(err) if status >= 500 => tracing::error!(
            target: "wide_event",
            request_id, method, route, status, duration_ms, user_agent, ip, extra,
            error.kind = err.kind, error.msg = err.message,
            "http_request"
        ),
        Some(err) => tracing::warn!(
            target: "wide_event",
            request_id, method, route, status, duration_ms, user_agent, ip, extra,
            error.kind = err.kind, error.msg = err.message,
            "http_request"
        ),
        None => tracing::info!(
            target: "wide_event",
            request_id, method, route, status, duration_ms, user_agent, ip, extra,
            "http_request"
        ),
    }

    response
}
