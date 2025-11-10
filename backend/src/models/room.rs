use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Room entity from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub room_type: String, // 'public' or 'private'
    pub owner_id: Uuid,
    pub max_members: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Room member entity from database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoomMember {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub role: String, // 'owner', 'admin', 'moderator', 'member'
    pub joined_at: DateTime<Utc>,
}

/// DTO for creating a room
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoomDto {
    #[validate(length(min = 3, max = 100, message = "Room name must be between 3-100 characters"))]
    pub name: String,
    
    #[validate(length(max = 500, message = "Description must not exceed 500 characters"))]
    pub description: Option<String>,
    
    pub room_type: String, // 'public' or 'private'
    
    #[validate(range(min = 2, max = 1000, message = "Max members must be between 2-1000"))]
    pub max_members: Option<i32>,
}

/// DTO for updating a room
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoomDto {
    #[validate(length(min = 3, max = 100, message = "Room name must be between 3-100 characters"))]
    pub name: Option<String>,
    
    #[validate(length(max = 500, message = "Description must not exceed 500 characters"))]
    pub description: Option<String>,
    
    pub room_type: Option<String>, // 'public' or 'private'
    
    #[validate(range(min = 2, max = 1000, message = "Max members must be between 2-1000"))]
    pub max_members: Option<i32>,
}

/// Room response (public data)
#[derive(Debug, Serialize, FromRow)]
pub struct RoomResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub room_type: String,
    pub owner_id: Uuid,
    pub max_members: Option<i32>,
    pub member_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Room member response with user info
#[derive(Debug, Serialize, FromRow)]
pub struct RoomMemberResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub status: String,
    pub joined_at: DateTime<Utc>,
}

/// Room with members response
#[derive(Debug, Serialize)]
pub struct RoomWithMembersResponse {
    pub room: RoomResponse,
    pub members: Vec<RoomMemberResponse>,
    pub is_member: bool,
    pub user_role: Option<String>,
}

impl From<Room> for RoomResponse {
    fn from(room: Room) -> Self {
        Self {
            id: room.id,
            name: room.name,
            description: room.description,
            room_type: room.room_type,
            owner_id: room.owner_id,
            max_members: room.max_members,
            member_count: 0, // Will be populated separately
            created_at: room.created_at,
            updated_at: room.updated_at,
        }
    }
}
