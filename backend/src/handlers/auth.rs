use actix_web::{web, HttpResponse};
use serde_json::json;
use crate::config::Config;
use crate::error::AppError;
use crate::models::user::{CreateUserDto, LoginDto};
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

    Ok(HttpResponse::Created().json(json!({
        "status": "success",
        "data": auth_response
    })))
}

/// POST /api/auth/login
/// Login user
pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    dto: web::Json<LoginDto>,
) -> Result<HttpResponse, AppError> {
    let auth_response = AuthService::login(&pool, &config, dto.into_inner()).await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": auth_response
    })))
}

/// GET /api/auth/me
/// Get current user info (requires authentication)
pub async fn get_me(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
) -> Result<HttpResponse, AppError> {
    let user = AuthService::get_me(&pool, auth_user.0).await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": { "user": user }
    })))
}

/// POST /api/auth/logout
/// Logout user (set status to offline)
pub async fn logout(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
) -> Result<HttpResponse, AppError> {
    AuthService::logout(&pool, auth_user.0).await?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Logged out successfully"
    })))
}
