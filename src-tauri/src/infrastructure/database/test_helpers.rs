#[cfg(test)]
pub async fn create_test_pool() -> sqlx::sqlite::SqlitePool {
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run test migrations");

    pool
}
