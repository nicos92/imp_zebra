use std::sync::Arc;
use sqlx::sqlite::SqlitePool;

pub struct AppState {
    pub db: Arc<SqlitePool>,
}

impl AppState {
    pub fn new(db: Arc<SqlitePool>) -> Self {
        Self { db }
    }
}
