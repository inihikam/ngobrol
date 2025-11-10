use actix_web::{web, HttpResponse};
use crate::config::Config;
use crate::error::AppError;
use crate::models::user::{CreateUserDto, LoginDto};
use crate::models::response::{success_response, created_response};
use crate::services::AuthService;
use crate::middleware::AuthUser;
use sqlx::PgPool;

/// POST /api/auth/register
/// Register a new user
pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    dto: web::Json<CreateUserDto>,
) -> Result<HttpResponse, AppError> {
    let auth_response = AuthService::register(&pool, &config, dto.into_inner()).await?;
    Ok(created_response(auth_response))
}

/// POST /api/auth/login
/// Login user
pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    dto: web::Json<LoginDto>,
) -> Result<HttpResponse, AppError> {
    let auth_response = AuthService::login(&pool, &config, dto.into_inner()).await?;
    Ok(success_response(auth_response))
}

/// GET /api/auth/me
/// Get current user info (requires authentication)
pub async fn get_me(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
) -> Result<HttpResponse, AppError> {
    let user = AuthService::get_me(&pool, auth_user.0).await?;
    Ok(success_response(user))
}

/// POST /api/auth/logout
/// Logout user (set status to offline)
pub async fn logout(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
) -> Result<HttpResponse, AppError> {
    AuthService::logout(&pool, auth_user.0).await?;
    Ok(success_response(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
