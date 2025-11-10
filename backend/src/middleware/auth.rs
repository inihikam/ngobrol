use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::config::Config;
use crate::error::AppError;
use crate::services::AuthService;
use sqlx::PgPool;

/// Middleware for JWT authentication
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: std::rc::Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: std::rc::Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Get Authorization header
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            Some(header) => {
                let header_str = match header.to_str() {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        let error = AppError::InvalidToken;
                        return Box::pin(async move { Err(error.into()) });
                    }
                };

                match crate::utils::jwt::extract_token_from_header(&header_str) {
                    Ok(t) => t,
                    Err(e) => {
                        return Box::pin(async move { Err(e.into()) });
                    }
                }
            }
            None => {
                let error = AppError::MissingToken;
                return Box::pin(async move { Err(error.into()) });
            }
        };

        // Get dependencies from app_data
        let pool = match req.app_data::<actix_web::web::Data<PgPool>>() {
            Some(p) => p.clone(),
            None => {
                let error = AppError::InternalError("Database pool not found".to_string());
                return Box::pin(async move { Err(error.into()) });
            }
        };

        let config = match req.app_data::<actix_web::web::Data<Config>>() {
            Some(c) => c.clone(),
            None => {
                let error = AppError::InternalError("Config not found".to_string());
                return Box::pin(async move { Err(error.into()) });
            }
        };

        let service = self.service.clone();

        Box::pin(async move {
            // Verify token and get user first
            let user = AuthService::verify_token(&pool, &config, &token).await?;

            // Insert user_id into request extensions BEFORE calling handler
            req.extensions_mut().insert(user.id);

            // Now call the handler
            let res = service.call(req).await?;

            Ok(res)
        })
    }
}
