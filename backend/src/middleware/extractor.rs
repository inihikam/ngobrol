use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpMessage};
use std::future::{ready, Ready};
use uuid::Uuid;
use crate::error::AppError;

/// Extractor for authenticated user ID
pub struct AuthUser(pub Uuid);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let user_id = req.extensions()
            .get::<uuid::Uuid>()
            .copied()
            .ok_or_else(|| AppError::MissingToken);

        ready(user_id.map(AuthUser).map_err(Into::into))
    }
}
