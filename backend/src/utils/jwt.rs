use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub email: String,    // User email
    pub username: String, // Username
}

/// Generate a JWT token for a user
pub fn generate_token(
    user_id: Uuid,
    email: &str,
    username: &str,
    secret: &str,
    expires_in_seconds: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiration = now + Duration::seconds(expires_in_seconds);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp(),
        iat: now.timestamp(),
        email: email.to_string(),
        username: username.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(token)
}

/// Verify and decode a JWT token
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Extract token from Authorization header
/// Expected format: "Bearer <token>"
pub fn extract_token_from_header(auth_header: &str) -> Result<String, AppError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InvalidToken);
    }

    let token = auth_header.trim_start_matches("Bearer ").trim();
    
    if token.is_empty() {
        return Err(AppError::InvalidToken);
    }

    Ok(token.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let username = "testuser";
        let secret = "test_secret_key_12345";
        let expires_in = 3600; // 1 hour

        // Generate token
        let token = generate_token(user_id, email, username, secret, expires_in)
            .expect("Failed to generate token");

        // Verify token
        let claims = verify_token(&token, secret).expect("Failed to verify token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.username, username);
    }

    #[test]
    fn test_verify_token_with_wrong_secret() {
        let user_id = Uuid::new_v4();
        let token = generate_token(user_id, "test@example.com", "testuser", "secret1", 3600)
            .expect("Failed to generate token");

        // Try to verify with wrong secret
        let result = verify_token(&token, "wrong_secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_header() {
        // Valid header
        let header = "Bearer abc123xyz";
        let token = extract_token_from_header(header).expect("Failed to extract token");
        assert_eq!(token, "abc123xyz");

        // Invalid header (missing Bearer)
        let invalid_header = "abc123xyz";
        let result = extract_token_from_header(invalid_header);
        assert!(result.is_err());

        // Invalid header (empty token)
        let empty_header = "Bearer ";
        let result = extract_token_from_header(empty_header);
        assert!(result.is_err());
    }
}
