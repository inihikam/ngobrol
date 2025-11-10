use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use crate::error::AppError;

/// Create a PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create database pool: {}", e)))?;

    log::info!("✅ Database connection pool created successfully");
    
    Ok(pool)
}

/// Test database connection
pub async fn test_connection(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Database connection test failed: {}", e)))?;

    log::info!("✅ Database connection test successful");
    
    Ok(())
}
