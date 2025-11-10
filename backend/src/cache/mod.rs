use redis::{Client, Connection};
use crate::error::AppError;

/// Create a Redis client
pub fn create_client(redis_url: &str) -> Result<Client, AppError> {
    Client::open(redis_url)
        .map_err(|e| AppError::RedisError(format!("Failed to create Redis client: {}", e)))
}

/// Get a connection from the Redis client
pub fn get_connection(client: &Client) -> Result<Connection, AppError> {
    client
        .get_connection()
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))
}

/// Test Redis connection
pub fn test_connection(client: &Client) -> Result<(), AppError> {
    let mut conn = get_connection(client)?;

    // Test basic operations
    redis::cmd("SET")
        .arg("test_key")
        .arg("test_value")
        .query::<()>(&mut conn)
        .map_err(|e| AppError::RedisError(format!("Redis SET failed: {}", e)))?;

    let _value: String = redis::cmd("GET")
        .arg("test_key")
        .query(&mut conn)
        .map_err(|e| AppError::RedisError(format!("Redis GET failed: {}", e)))?;

    redis::cmd("DEL")
        .arg("test_key")
        .query::<()>(&mut conn)
        .map_err(|e| AppError::RedisError(format!("Redis DEL failed: {}", e)))?;

    log::info!("✅ Redis connection test successful");
    Ok(())
}
