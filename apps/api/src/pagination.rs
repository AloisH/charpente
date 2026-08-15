//! Cursor pagination + list-param conventions shared by every list endpoint.
//! Because `Page<T>` lives in the OpenAPI spec, the generated client gets a
//! matching generic type (and infinite-query helpers) for free.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

pub const DEFAULT_LIMIT: i64 = 25;
pub const MAX_LIMIT: i64 = 100;

/// `?cursor=<uuid>&limit=<n>` — cursor is the id of the last item of the
/// previous page. UUIDv7 ids are time-ordered, so "id < cursor" pages
/// newest-first without a separate sort column.
#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CursorParams {
    pub cursor: Option<Uuid>,
    /// Page size, 1..=100 (default 25).
    pub limit: Option<i64>,
}

impl CursorParams {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    /// Pass as `cursor` to fetch the next page; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Uuid>,
    pub has_more: bool,
}

impl<T> Page<T> {
    /// Build a page from `limit + 1` fetched rows; `id_of` extracts the cursor.
    pub fn from_rows(mut rows: Vec<T>, limit: i64, id_of: impl Fn(&T) -> Uuid) -> Self {
        let limit = usize::try_from(limit.max(0)).unwrap_or(usize::MAX);
        let has_more = rows.len() > limit;
        rows.truncate(limit);
        let next_cursor = has_more.then(|| rows.last().map(&id_of)).flatten();
        Self {
            items: rows,
            next_cursor,
            has_more,
        }
    }
}
