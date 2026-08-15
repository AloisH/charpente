//! Integration tests: every test gets its own transactional database via
//! `#[sqlx::test]` (migrations from ./migrations run automatically), and
//! exercises the real production router through `tower::ServiceExt::oneshot`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use api::config::{AppEnv, Config};
use api::mailer::memory::MemoryMailer;
use api::state::AppState;
use api::storage::memory::MemoryStorage;

fn test_config() -> Config {
    Config {
        app_env: AppEnv::Dev,
        app_host: "127.0.0.1".into(),
        app_port: 0,
        app_public_url: None,
        database_url: String::new(),
        // 64 zero bytes, base64-encoded — fine for tests.
        session_key: base64_key(),
        s3_endpoint: "http://storage.test".into(),
        s3_public_endpoint: None,
        s3_region: "us-east-1".into(),
        s3_bucket: "test".into(),
        s3_access_key: "test".into(),
        s3_secret_key: "test".into(),
        seed_admin_email: Some("admin@example.com".into()),
        seed_admin_password: Some("admin-password-123".into()),
        smtp_url: None,
        mail_from: "test <no-reply@test>".into(),
        mail_base_url: Some("http://app.test".into()),
    }
}

fn base64_key() -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode([7u8; 64])
}

async fn app(pool: PgPool) -> Router {
    app_with_mailer(pool).await.0
}

/// Variant handing back the in-memory mailer so tests can read sent mail.
async fn app_with_mailer(pool: PgPool) -> (Router, Arc<MemoryMailer>) {
    let config = Arc::new(test_config());
    let mailer = Arc::new(MemoryMailer::default());
    let state = AppState {
        pool,
        config,
        storage: Arc::new(MemoryStorage::default()),
        mailer: Arc::clone(&mailer) as _,
    };
    let router = api::app::build(state).await.expect("router builds");
    (router, mailer)
}

/// Request builder that always carries ConnectInfo (the rate limiter needs a
/// peer IP) and optionally a session cookie.
fn req(method: &str, path: &str, cookie: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .extension(ConnectInfo("127.0.0.1:9999".parse::<SocketAddr>().unwrap()));
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

fn session_cookie(response: &axum::response::Response) -> String {
    response
        .headers()
        .get(header::SET_COOKIE)
        .expect("session cookie set")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

async fn register(router: &Router, email: &str) -> (String, Value) {
    let response = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/register",
            None,
            Some(json!({
                "email": email,
                "password": "correct-horse-battery",
                "display_name": "Test User"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let cookie = session_cookie(&response);
    (cookie, body_json(response).await)
}

#[sqlx::test]
async fn health_endpoints(pool: PgPool) {
    let router = app(pool).await;
    let live = router
        .clone()
        .oneshot(req("GET", "/health/live", None, None))
        .await
        .unwrap();
    assert_eq!(live.status(), StatusCode::NO_CONTENT);
    let ready = router
        .oneshot(req("GET", "/health/ready", None, None))
        .await
        .unwrap();
    assert_eq!(ready.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn register_login_me_logout_flow(pool: PgPool) {
    let router = app(pool).await;

    let (cookie, user) = register(&router, "alice@example.com").await;
    assert_eq!(user["email"], "alice@example.com");
    assert_eq!(user["role"], "user");

    // Session works
    let me = router
        .clone()
        .oneshot(req("GET", "/api/v1/auth/me", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);

    // Fresh login
    let login = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({"email": "Alice@Example.com", "password": "correct-horse-battery"})),
        ))
        .await
        .unwrap();
    assert_eq!(
        login.status(),
        StatusCode::OK,
        "login is email-case-insensitive"
    );
    let cookie2 = session_cookie(&login);

    // Logout kills the session
    let logout = router
        .clone()
        .oneshot(req("POST", "/api/v1/auth/logout", Some(&cookie2), None))
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    let me_after = router
        .oneshot(req("GET", "/api/v1/auth/me", Some(&cookie2), None))
        .await
        .unwrap();
    assert_eq!(me_after.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_with_wrong_password_is_generic_401(pool: PgPool) {
    let router = app(pool).await;
    register(&router, "bob@example.com").await;

    let response = router
        .oneshot(req(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({"email": "bob@example.com", "password": "wrong"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = body_json(response).await;
    assert_eq!(body["code"], "invalid_credentials");
}

#[sqlx::test]
async fn duplicate_email_conflicts(pool: PgPool) {
    let router = app(pool).await;
    register(&router, "carol@example.com").await;

    let response = router
        .oneshot(req(
            "POST",
            "/api/v1/auth/register",
            None,
            Some(json!({
                "email": "CAROL@example.com",
                "password": "another-password-1",
                "display_name": "Carol Again"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "emails are case-insensitive"
    );
}

#[sqlx::test]
async fn register_validation_shape(pool: PgPool) {
    let router = app(pool).await;
    let response = router
        .oneshot(req(
            "POST",
            "/api/v1/auth/register",
            None,
            Some(json!({"email": "not-an-email", "password": "short", "display_name": ""})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/problem+json"
    );
    let body = body_json(response).await;
    insta::assert_json_snapshot!(body);
}

#[sqlx::test]
async fn admin_routes_are_guarded(pool: PgPool) {
    let router = app(pool.clone()).await;

    // Anonymous → 401
    let anon = router
        .clone()
        .oneshot(req("GET", "/api/v1/admin/users", None, None))
        .await
        .unwrap();
    assert_eq!(anon.status(), StatusCode::UNAUTHORIZED);

    // Plain user → 403
    let (cookie, user) = register(&router, "dave@example.com").await;
    let forbidden = router
        .clone()
        .oneshot(req("GET", "/api/v1/admin/users", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    // Promote to admin directly in the DB, then the same session may pass.
    let user_id: uuid::Uuid = serde_json::from_value(user["id"].clone()).unwrap();
    sqlx::query!("UPDATE users SET role = 'admin' WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .unwrap();

    let allowed = router
        .oneshot(req("GET", "/api/v1/admin/users", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(allowed.status(), StatusCode::OK);
    let body = body_json(allowed).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["has_more"], false);
}

#[sqlx::test]
async fn set_role_writes_audit_log(pool: PgPool) {
    let router = app(pool.clone()).await;
    let (admin_cookie, admin) = register(&router, "root@example.com").await;
    let admin_id: uuid::Uuid = serde_json::from_value(admin["id"].clone()).unwrap();
    sqlx::query!("UPDATE users SET role = 'admin' WHERE id = $1", admin_id)
        .execute(&pool)
        .await
        .unwrap();

    let (_, target) = register(&router, "eve@example.com").await;
    let target_id = target["id"].as_str().unwrap();

    let response = router
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/admin/users/{target_id}/role"),
            Some(&admin_cookie),
            Some(json!({"role": "admin"})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let audit = router
        .oneshot(req(
            "GET",
            "/api/v1/admin/audit-log",
            Some(&admin_cookie),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(audit.status(), StatusCode::OK);
    let body = body_json(audit).await;
    assert_eq!(body["items"][0]["action"], "user.set_role");
}

#[sqlx::test]
async fn upload_flow_presign_complete_download(pool: PgPool) {
    let router = app(pool).await;
    let (cookie, _) = register(&router, "frank@example.com").await;

    // Declare
    let created = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/uploads",
            Some(&cookie),
            Some(json!({"filename": "cat.png", "content_type": "image/png", "size_bytes": 1024})),
        ))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let body = body_json(created).await;
    assert!(
        body["put_url"]
            .as_str()
            .unwrap()
            .starts_with("http://storage.test/put/")
    );
    let id = body["upload"]["id"].as_str().unwrap().to_owned();
    assert_eq!(body["upload"]["state"], "pending");

    // Complete (MemoryStorage marks the object present at presign time)
    let completed = router
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/uploads/{id}/complete"),
            Some(&cookie),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(completed.status(), StatusCode::OK);
    assert_eq!(body_json(completed).await["state"], "complete");

    // List + download
    let list = router
        .clone()
        .oneshot(req("GET", "/api/v1/uploads", Some(&cookie), None))
        .await
        .unwrap();
    let list_body = body_json(list).await;
    assert_eq!(list_body["items"].as_array().unwrap().len(), 1);

    let download = router
        .oneshot(req(
            "GET",
            &format!("/api/v1/uploads/{id}/download"),
            Some(&cookie),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(download.status(), StatusCode::OK);
}

#[sqlx::test]
async fn upload_rejects_disallowed_content_type(pool: PgPool) {
    let router = app(pool).await;
    let (cookie, _) = register(&router, "grace@example.com").await;

    let response = router
        .oneshot(req(
            "POST",
            "/api/v1/uploads",
            Some(&cookie),
            Some(json!({
                "filename": "evil.exe",
                "content_type": "application/x-msdownload",
                "size_bytes": 1024
            })),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(response).await;
    assert!(
        body["errors"]["content_type"][0]
            .as_str()
            .unwrap()
            .starts_with("must be one of")
    );
}

#[sqlx::test]
async fn account_export_and_erasure(pool: PgPool) {
    let router = app(pool.clone()).await;
    let (cookie, _) = register(&router, "henry@example.com").await;

    let export = router
        .clone()
        .oneshot(req("GET", "/api/v1/account/export", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(export.status(), StatusCode::OK);
    let body = body_json(export).await;
    assert_eq!(body["user"]["email"], "henry@example.com");

    let erased = router
        .clone()
        .oneshot(req("DELETE", "/api/v1/account", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(erased.status(), StatusCode::NO_CONTENT);

    let count = sqlx::query_scalar!("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, Some(0));

    // The old session is dead.
    let me = router
        .oneshot(req("GET", "/api/v1/auth/me", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn unknown_api_route_is_problem_json_404(pool: PgPool) {
    let router = app(pool).await;
    let response = router
        .oneshot(req("GET", "/api/v1/nope", None, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/problem+json"
    );
}

#[sqlx::test]
async fn wide_event_request_id_header_is_returned(pool: PgPool) {
    let router = app(pool).await;
    let response = router
        .oneshot(req("GET", "/health/live", None, None))
        .await
        .unwrap();
    assert!(response.headers().contains_key("x-request-id"));
}

/// The OpenAPI spec itself is snapshotted: any change to the API contract
/// shows up as a reviewable diff.
#[test]
fn openapi_spec_snapshot() {
    let doc = api::openapi::document();
    let json: Value = serde_json::from_str(&doc.to_json().unwrap()).unwrap();
    insta::assert_json_snapshot!(json);
}

// ── Email verification ───────────────────────────────────────────

/// Pull the raw token out of the emailed link (`…/verify-email?token=<t>`).
fn mail_token(body: &str) -> String {
    let start = body.find("token=").expect("mail contains a token link") + "token=".len();
    body[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

#[sqlx::test]
async fn email_verification_flow(pool: PgPool) {
    let (router, mailer) = app_with_mailer(pool).await;
    let (cookie, user) = register(&router, "vera@example.com").await;
    assert_eq!(user["email_verified"], json!(false));

    let sent = mailer.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "vera@example.com");
    let token = mail_token(&sent[0].body);
    assert!(sent[0].body.contains("http://app.test/verify-email?token="));

    // Unknown token: 404.
    let bad = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/verify-email",
            None,
            Some(json!({ "token": "nope" })),
        ))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::NOT_FOUND);

    // The real token verifies — no session required (any browser may open it).
    let ok = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/verify-email",
            None,
            Some(json!({ "token": token })),
        ))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    // Single-use.
    let again = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/verify-email",
            None,
            Some(json!({ "token": token })),
        ))
        .await
        .unwrap();
    assert_eq!(again.status(), StatusCode::NOT_FOUND);

    let me = router
        .oneshot(req("GET", "/api/v1/auth/me", Some(&cookie), None))
        .await
        .unwrap();
    assert_eq!(body_json(me).await["email_verified"], json!(true));
}

#[sqlx::test]
async fn resend_verification_rotates_tokens(pool: PgPool) {
    let (router, mailer) = app_with_mailer(pool).await;
    let (cookie, _) = register(&router, "rene@example.com").await;

    let resent = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/resend-verification",
            Some(&cookie),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resent.status(), StatusCode::NO_CONTENT);

    let sent = mailer.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 2);
    let (first, second) = (mail_token(&sent[0].body), mail_token(&sent[1].body));
    assert_ne!(first, second);

    // Re-issuing invalidated the first token.
    let stale = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/verify-email",
            None,
            Some(json!({ "token": first })),
        ))
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::NOT_FOUND);

    let ok = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/verify-email",
            None,
            Some(json!({ "token": second })),
        ))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    // Already verified: resending is a conflict.
    let conflict = router
        .oneshot(req(
            "POST",
            "/api/v1/auth/resend-verification",
            Some(&cookie),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
}

// ── Password reset ───────────────────────────────────────────────

/// The forgot-password work happens off-request (anti-enumeration), so tests
/// poll the in-memory mailbox instead of reading it synchronously.
async fn wait_for_mail(mailer: &MemoryMailer, count: usize) -> Vec<api::mailer::memory::SentMail> {
    for _ in 0..200 {
        {
            let sent = mailer.sent.lock().unwrap();
            if sent.len() >= count {
                return sent.clone();
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    panic!("expected at least {count} mails");
}

#[sqlx::test]
async fn password_reset_flow(pool: PgPool) {
    let (router, mailer) = app_with_mailer(pool).await;
    register(&router, "resette@example.com").await;

    let forgot = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/forgot-password",
            None,
            Some(json!({ "email": "resette@example.com" })),
        ))
        .await
        .unwrap();
    assert_eq!(forgot.status(), StatusCode::NO_CONTENT);

    // Mail 0 is the registration's verification email; mail 1 is the reset.
    let sent = wait_for_mail(&mailer, 2).await;
    assert!(sent[1].subject.contains("Reset"));
    let token = mail_token(&sent[1].body);
    assert!(sent[1].body.contains("/reset-password?token="));

    // Too-short password: 422, token survives (validation runs first).
    let weak = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/reset-password",
            None,
            Some(json!({ "token": token, "password": "short" })),
        ))
        .await
        .unwrap();
    assert_eq!(weak.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let ok = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/reset-password",
            None,
            Some(json!({ "token": token, "password": "brand-new-password" })),
        ))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    // Single-use.
    let reuse = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/reset-password",
            None,
            Some(json!({ "token": token, "password": "brand-new-password" })),
        ))
        .await
        .unwrap();
    assert_eq!(reuse.status(), StatusCode::NOT_FOUND);

    // Old password is dead, the new one works, and receiving the reset mail
    // proved ownership: the account is now verified.
    let old = router
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({ "email": "resette@example.com", "password": "correct-horse-battery" })),
        ))
        .await
        .unwrap();
    assert_eq!(old.status(), StatusCode::UNAUTHORIZED);

    let new = router
        .oneshot(req(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({ "email": "resette@example.com", "password": "brand-new-password" })),
        ))
        .await
        .unwrap();
    assert_eq!(new.status(), StatusCode::OK);
    assert_eq!(body_json(new).await["email_verified"], json!(true));
}

#[sqlx::test]
async fn forgot_password_never_reveals_accounts(pool: PgPool) {
    let (router, mailer) = app_with_mailer(pool).await;

    let unknown = router
        .oneshot(req(
            "POST",
            "/api/v1/auth/forgot-password",
            None,
            Some(json!({ "email": "ghost@example.com" })),
        ))
        .await
        .unwrap();
    // Identical answer to the known-account case…
    assert_eq!(unknown.status(), StatusCode::NO_CONTENT);

    // …and no mail goes out.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(mailer.sent.lock().unwrap().len(), 0);
}
