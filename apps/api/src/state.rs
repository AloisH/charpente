use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
use crate::mailer::Mailer;
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub storage: Arc<dyn Storage>,
    pub mailer: Arc<dyn Mailer>,
}
