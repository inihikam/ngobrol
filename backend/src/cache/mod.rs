use redis::{Client, Connection};
use crate::error::AppError;

/// Create a Redis client
pub fn create_client(redis_url: &str) -> Result<Client, AppError> {
    let client = Client::open(redis_url)
        .map_err(|e| AppError::CacheError(format!("Failed to create Redis client: {}", e)))?;

    log::info!("✅ Redis client created successfully");
    
    Ok(client)
}

/// Get a Redis connection from client
pub fn get_connection(client: &Client) -> Result<Connection, AppError> {
    let conn = client.get_connection()
        .map_err(|e| AppError::CacheError(format!("Failed to get Redis connection: {}", e)))?;

    Ok(conn)
}

/// Test Redis connection
pub fn test_connection(client: &Client) -> Result<(), AppError> {
    use redis::Commands;
    
    let mut conn = get_connection(client)?;
    
    // Simple SET/GET test
    conn.set::<&str, &str, ()>("test_key", "test_value")
        .map_err(|e| AppError::CacheError(format!("Redis SET failed: {}", e)))?;
    
    let _: String = conn.get("test_key")
        .map_err(|e| AppError::CacheError(format!("Redis GET failed: {}", e)))?;
    
    // Clean up test key
    let _: () = conn.del("test_key")
        .map_err(|e| AppError::CacheError(format!("Redis DEL failed: {}", e)))?;

    log::info!("✅ Redis connection test successful");
    
    Ok(())
}
