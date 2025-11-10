use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;
use crate::error::{AppError, ValidationErrors};
use crate::models::room::{CreateRoomDto, UpdateRoomDto, RoomResponse, RoomMemberResponse, RoomWithMembersResponse};
use crate::repositories::RoomRepository;

pub struct RoomService;

impl RoomService {
    /// Create a new room
    pub async fn create_room(
        pool: &PgPool,
        dto: CreateRoomDto,
        owner_id: Uuid,
    ) -> Result<RoomResponse, AppError> {
        // Validate input
        dto.validate()
            .map_err(|_| {
                let mut errors = ValidationErrors::new();
                errors.add_field_error("input", "Invalid room data");
                AppError::ValidationError(errors)
            })?;

        // Check if room name already exists
        if RoomRepository::name_exists(pool, &dto.name).await? {
            return Err(AppError::RoomNameExists);
        }

        // Create room
        let room = RoomRepository::create(pool, &dto, owner_id).await?;

        // Add creator as owner
        RoomRepository::add_member(pool, room.id, owner_id, "owner").await?;

        // Get member count
        let member_count = RoomRepository::count_members(pool, room.id).await?;

        let mut room_response = RoomResponse::from(room);
        room_response.member_count = member_count;

        Ok(room_response)
    }

    /// Get list of rooms accessible by user
    pub async fn get_rooms(
        pool: &PgPool,
        user_id: Uuid,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<RoomResponse>, i64), AppError> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        // Get rooms with member counts already included
        let rooms = RoomRepository::list_rooms(pool, offset, limit).await?;

        // Get total count
        let total = RoomRepository::count_rooms(pool, user_id).await?;

        Ok((rooms, total))
    }

    /// Get room details with members
    pub async fn get_room(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<RoomWithMembersResponse, AppError> {
        // Get room
        let room = RoomRepository::find_by_id(pool, room_id).await?;

        // Check if user has access (public room or is member)
        let is_member = RoomRepository::is_member(pool, room_id, user_id).await?;

        if room.room_type == "private" && !is_member {
            return Err(AppError::PrivateNoAccess);
        }

        // Get members
        let members = RoomRepository::get_members(pool, room_id).await?;

        // Get user role
        let user_role = RoomRepository::get_user_role(pool, room_id, user_id).await?;

        // Get member count
        let member_count = members.len() as i64;

        let mut room_response = RoomResponse::from(room);
        room_response.member_count = member_count;

        Ok(RoomWithMembersResponse {
            room: room_response,
            members,
            is_member,
            user_role,
        })
    }

    /// Update room (only owner/admin can update)
    pub async fn update_room(
        pool: &PgPool,
        room_id: Uuid,
        dto: UpdateRoomDto,
        user_id: Uuid,
    ) -> Result<RoomResponse, AppError> {
        // Validate input
        dto.validate()
            .map_err(|_| {
                let mut errors = ValidationErrors::new();
                errors.add_field_error("input", "Invalid room data");
                AppError::ValidationError(errors)
            })?;

        // Check if room exists
        let _room = RoomRepository::find_by_id(pool, room_id).await?;

        // Check permissions (owner or admin)
        let role = RoomRepository::get_user_role(pool, room_id, user_id).await?;
        
        match role.as_deref() {
            Some("owner") | Some("admin") => {
                // Update room
                let updated_room = RoomRepository::update(pool, room_id, &dto).await?;
                
                // Get member count
                let member_count = RoomRepository::count_members(pool, room_id).await?;
                
                let mut room_response = RoomResponse::from(updated_room);
                room_response.member_count = member_count;
                
                Ok(room_response)
            }
            _ => Err(AppError::InsufficientPermissions),
        }
    }

    /// Delete room (only owner can delete)
    pub async fn delete_room(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // Check if room exists
        let _room = RoomRepository::find_by_id(pool, room_id).await?;

        // Check if user is owner
        let role = RoomRepository::get_user_role(pool, room_id, user_id).await?;
        
        if role.as_deref() != Some("owner") {
            return Err(AppError::OwnerRequired);
        }

        // Delete room (cascade will delete members and messages)
        RoomRepository::delete(pool, room_id).await?;

        Ok(())
    }

    /// Join a room
    pub async fn join_room(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<RoomMemberResponse, AppError> {
        // Check if room exists
        let room = RoomRepository::find_by_id(pool, room_id).await?;

        // Check if already a member
        if RoomRepository::is_member(pool, room_id, user_id).await? {
            return Err(AppError::AlreadyJoined);
        }

        // Check if room is full
        if let Some(max_members) = room.max_members {
            let member_count = RoomRepository::count_members(pool, room_id).await?;
            if member_count >= max_members as i64 {
                return Err(AppError::RoomFull);
            }
        }

        // Check if private room
        if room.room_type == "private" {
            return Err(AppError::PrivateNoAccess);
        }

        // Add as member
        RoomRepository::add_member(pool, room_id, user_id, "member").await?;

        // Get updated member info
        let members = RoomRepository::get_members(pool, room_id).await?;
        let member = members
            .into_iter()
            .find(|m| m.user_id == user_id)
            .ok_or(AppError::InternalError("Failed to retrieve member info".to_string()))?;

        Ok(member)
    }

    /// Leave a room
    pub async fn leave_room(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // Check if room exists
        let _room = RoomRepository::find_by_id(pool, room_id).await?;

        // Check if user is owner
        let role = RoomRepository::get_user_role(pool, room_id, user_id).await?;
        if role.as_deref() == Some("owner") {
            return Err(AppError::OwnerRequired);
        }

        // Remove member
        RoomRepository::remove_member(pool, room_id, user_id).await?;

        Ok(())
    }

    /// Get room members
    pub async fn get_members(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<RoomMemberResponse>, AppError> {
        // Check if room exists
        let room = RoomRepository::find_by_id(pool, room_id).await?;

        // Check if user has access (member or public room)
        let is_member = RoomRepository::is_member(pool, room_id, user_id).await?;

        if room.room_type == "private" && !is_member {
            return Err(AppError::PrivateNoAccess);
        }

        // Get members
        let members = RoomRepository::get_members(pool, room_id).await?;

        Ok(members)
    }
}
