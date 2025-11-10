use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use crate::middleware::AuthUser;
use crate::models::room::{CreateRoomDto, UpdateRoomDto};
use crate::models::response::{success_response, created_response, paginated_response, no_content_response};
use crate::services::RoomService;

/// Query params for listing rooms
#[derive(Deserialize)]
pub struct ListRoomsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// GET /api/rooms
/// Get list of rooms accessible by user
pub async fn list_rooms(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    query: web::Query<ListRoomsQuery>,
) -> Result<HttpResponse, AppError> {
    let (rooms, total) = RoomService::get_rooms(
        &pool,
        auth_user.0,
        query.page,
        query.per_page,
    )
    .await?;

    Ok(paginated_response(rooms, query.page, query.per_page, total as u64))
}

/// POST /api/rooms
/// Create a new room
pub async fn create_room(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    dto: web::Json<CreateRoomDto>,
) -> Result<HttpResponse, AppError> {
    let room = RoomService::create_room(&pool, dto.into_inner(), auth_user.0).await?;
    Ok(created_response(room))
}

/// GET /api/rooms/:id
/// Get room details with members
pub async fn get_room(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    room_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let room = RoomService::get_room(&pool, *room_id, auth_user.0).await?;
    Ok(success_response(room))
}

/// PUT /api/rooms/:id
/// Update room (owner/admin only)
pub async fn update_room(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    room_id: web::Path<Uuid>,
    dto: web::Json<UpdateRoomDto>,
) -> Result<HttpResponse, AppError> {
    let room = RoomService::update_room(&pool, *room_id, dto.into_inner(), auth_user.0).await?;
    Ok(success_response(room))
}

/// DELETE /api/rooms/:id
/// Delete room (owner only)
pub async fn delete_room(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    room_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    RoomService::delete_room(&pool, *room_id, auth_user.0).await?;
    Ok(no_content_response())
}

/// POST /api/rooms/:id/join
/// Join a public room
pub async fn join_room(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    room_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let member = RoomService::join_room(&pool, *room_id, auth_user.0).await?;
    Ok(created_response(member))
}

/// POST /api/rooms/:id/leave
/// Leave a room (except owner)
pub async fn leave_room(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    room_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    RoomService::leave_room(&pool, *room_id, auth_user.0).await?;
    Ok(no_content_response())
}

/// GET /api/rooms/:id/members
/// Get room members
pub async fn get_members(
    pool: web::Data<PgPool>,
    auth_user: AuthUser,
    room_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let members = RoomService::get_members(&pool, *room_id, auth_user.0).await?;
    Ok(success_response(members))
}
