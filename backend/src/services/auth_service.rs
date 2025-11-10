use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;
use crate::config::Config;
use crate::error::AppError;
use crate::models::user::{User, CreateUserDto, LoginDto, AuthResponse, UserResponse};
use crate::repositories::UserRepository;
use crate::utils::{password, jwt};

pub struct AuthService;

impl AuthService {
    /// Register a new user
    pub async fn register(
        pool: &PgPool,
        config: &Config,
        dto: CreateUserDto,
    ) -> Result<AuthResponse, AppError> {
        // Validate input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Check if email already exists
        if UserRepository::email_exists(pool, &dto.email).await? {
            return Err(AppError::DuplicateEntry("Email already registered".to_string()));
        }

        // Check if username already exists
        if UserRepository::username_exists(pool, &dto.username).await? {
            return Err(AppError::DuplicateEntry("Username already taken".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&dto.password)?;

        // Create user in database
        let user = UserRepository::create(pool, &dto, &password_hash).await?;

        // Generate JWT token
        let token = jwt::generate_token(
            user.id,
            &user.email,
            &user.username,
            &config.jwt_secret,
            config.jwt_expires_in,
        )?;

        Ok(AuthResponse {
            user: user.into(),
            token,
        })
    }

    /// Login user
    pub async fn login(
        pool: &PgPool,
        config: &Config,
        dto: LoginDto,
    ) -> Result<AuthResponse, AppError> {
        // Validate input
        dto.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // Find user by email
        let user = UserRepository::find_by_email(pool, &dto.email)
            .await
            .map_err(|_| AppError::InvalidCredentials)?;

        // Verify password
        let is_valid = password::verify_password(&dto.password, &user.password_hash)?;
        
        if !is_valid {
            return Err(AppError::InvalidCredentials);
        }

        // Update user status to online
        UserRepository::update_status(pool, user.id, "online").await?;

        // Generate JWT token
        let token = jwt::generate_token(
            user.id,
            &user.email,
            &user.username,
            &config.jwt_secret,
            config.jwt_expires_in,
        )?;

        Ok(AuthResponse {
            user: user.into(),
            token,
        })
    }

    /// Get current user from token
    pub async fn get_me(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<UserResponse, AppError> {
        let user = UserRepository::find_by_id(pool, user_id).await?;
        Ok(user.into())
    }

    /// Logout user (update status to offline)
    pub async fn logout(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        UserRepository::update_status(pool, user_id, "offline").await?;
        Ok(())
    }

    /// Verify JWT token and return user
    pub async fn verify_token(
        pool: &PgPool,
        config: &Config,
        token: &str,
    ) -> Result<User, AppError> {
        // Verify and decode token
        let claims = jwt::verify_token(token, &config.jwt_secret)?;

        // Parse user ID from claims
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidToken)?;

        // Fetch user from database
        let user = UserRepository::find_by_id(pool, user_id).await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a test database setup
    // For now, they are placeholders for the test structure

    #[tokio::test]
    #[ignore] // Ignore until test database is set up
    async fn test_register_success() {
        // TODO: Setup test database
        // TODO: Test successful registration
    }

    #[tokio::test]
    #[ignore]
    async fn test_register_duplicate_email() {
        // TODO: Test duplicate email error
    }

    #[tokio::test]
    #[ignore]
    async fn test_login_success() {
        // TODO: Test successful login
    }

    #[tokio::test]
    #[ignore]
    async fn test_login_invalid_credentials() {
        // TODO: Test invalid credentials error
    }
}
