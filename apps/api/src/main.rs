use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;

use api::config::Config;
use api::state::AppState;
use api::{app, db, mailer, openapi, seed, storage, telemetry};

fn main() -> anyhow::Result<()> {
    // `dump-openapi` must work without env vars, a database, or a runtime —
    // it runs in CI's generated-check and inside the dev watch loop.
    let command = std::env::args().nth(1);
    if command.as_deref() == Some("dump-openapi") {
        return dump_openapi();
    }

    dotenvy::dotenv().ok();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to start tokio runtime")?
        .block_on(async_main(command))
}

fn dump_openapi() -> anyhow::Result<()> {
    let path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "openapi.json".to_owned());
    let doc = openapi::document();
    let json = doc
        .to_pretty_json()
        .context("failed to serialize OpenAPI document")?;
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("failed to write {path}"))?;
    println!("wrote {path}");
    Ok(())
}

async fn async_main(command: Option<String>) -> anyhow::Result<()> {
    let config = Config::from_env()?;
    telemetry::init(config.app_env);

    let pool = db::connect(&config.database_url)
        .await
        .context("database connection failed")?;
    db::migrate(&pool).await.context("migrations failed")?;

    if command.as_deref() == Some("seed") {
        return seed::seed_admin(&pool, &config).await;
    }

    let storage = Arc::new(storage::S3Storage::from_config(&config));
    let mailer = mailer::from_config(&config).context("mailer configuration failed")?;
    let state = AppState {
        pool,
        config: Arc::new(config.clone()),
        storage,
        mailer,
    };
    let router = app::build(state).await?;

    let addr = format!("{}:{}", config.app_host, config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(
        target: "wide_event",
        operation = "startup",
        addr,
        version = env!("CARGO_PKG_VERSION"),
        "listening"
    );

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("server error")?;

    Ok(())
}

/// Resolve on SIGINT (ctrl-c) or SIGTERM (docker stop), letting in-flight
/// requests finish.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!(target: "wide_event", operation = "shutdown", "shutting down");
}
