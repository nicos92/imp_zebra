use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<SqlitePool>,
}

impl AppState {
    pub fn new(db: Arc<SqlitePool>) -> Self {
        Self { db }
    }
}
