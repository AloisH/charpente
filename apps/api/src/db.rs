//! Connection pool + migrations. SQLx's Postgres migrator already serializes
//! concurrent runs with an advisory lock, so booting several replicas at once
//! is safe.

use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        // Fail fast on a broken statement instead of hanging a request —
        // applied per connection so every pooled session gets the timeout.
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::Executor::execute(conn, "SET statement_timeout = '30s'").await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
