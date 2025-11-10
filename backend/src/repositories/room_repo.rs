use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use crate::models::room::{Room, RoomMember, CreateRoomDto, UpdateRoomDto, RoomResponse, RoomMemberResponse};

pub struct RoomRepository;

impl RoomRepository {
    /// Create a new room
    pub async fn create(
        pool: &PgPool,
        dto: &CreateRoomDto,
        owner_id: Uuid,
    ) -> Result<Room, AppError> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            INSERT INTO rooms (name, description, room_type, owner_id, max_members)
            VALUES ($1, $2, $3::room_type, $4, $5)
            RETURNING id, name, description, room_type::text as room_type, owner_id, max_members, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.description)
        .bind(&dto.room_type)
        .bind(owner_id)
        .bind(dto.max_members)
        .fetch_one(pool)
        .await?;

        Ok(room)
    }

    /// Find room by ID
    pub async fn find_by_id(pool: &PgPool, room_id: Uuid) -> Result<Room, AppError> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            SELECT id, name, description, room_type::text as room_type, owner_id, max_members, created_at, updated_at
            FROM rooms WHERE id = $1
            "#,
        )
        .bind(room_id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::RoomNotFound)?;

        Ok(room)
    }

    /// List rooms with pagination
    pub async fn list_rooms(
        pool: &PgPool,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<RoomResponse>, AppError> {
        let rooms = sqlx::query_as::<_, RoomResponse>(
            r#"
            SELECT 
                r.id, 
                r.name, 
                r.description, 
                r.room_type::text as room_type,
                r.owner_id, 
                r.max_members, 
                r.created_at, 
                r.updated_at,
                COUNT(rm.id) as member_count
            FROM rooms r
            LEFT JOIN room_members rm ON r.id = rm.room_id
            GROUP BY r.id
            ORDER BY r.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rooms)
    }

    /// Count total rooms accessible by user
    pub async fn count_rooms(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT r.id)
            FROM rooms r
            LEFT JOIN room_members rm ON r.id = rm.room_id AND rm.user_id = $1
            WHERE r.room_type = 'public' OR rm.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Update room
    pub async fn update(
        pool: &PgPool,
        room_id: Uuid,
        updates: &UpdateRoomDto,
    ) -> Result<Room, AppError> {
        let mut query = String::from("UPDATE rooms SET ");
        let mut params: Vec<String> = vec![];
        let mut param_count = 1;

        if let Some(_) = &updates.name {
            params.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if let Some(_) = &updates.description {
            params.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if let Some(_) = &updates.room_type {
            params.push(format!("room_type = ${}::room_type", param_count));
            param_count += 1;
        }
        if let Some(_) = &updates.max_members {
            params.push(format!("max_members = ${}", param_count));
            param_count += 1;
        }

        if params.is_empty() {
            return Self::find_by_id(pool, room_id).await;
        }

        query.push_str(&params.join(", "));
        query.push_str(&format!(
            " WHERE id = ${} RETURNING id, name, description, room_type::text as room_type, owner_id, max_members, created_at, updated_at",
            param_count
        ));

        let mut sqlx_query = sqlx::query_as::<_, Room>(&query);

        if let Some(ref name) = updates.name {
            sqlx_query = sqlx_query.bind(name);
        }
        if let Some(ref description) = updates.description {
            sqlx_query = sqlx_query.bind(description);
        }
        if let Some(ref room_type) = updates.room_type {
            sqlx_query = sqlx_query.bind(room_type);
        }
        if let Some(max_members) = updates.max_members {
            sqlx_query = sqlx_query.bind(max_members);
        }

        sqlx_query = sqlx_query.bind(room_id);

        let room = sqlx_query.fetch_one(pool).await?;
        Ok(room)
    }

    /// Delete room (only owner can delete)
    pub async fn delete(pool: &PgPool, room_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM rooms WHERE id = $1
            "#,
        )
        .bind(room_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::RoomNotFound);
        }

        Ok(())
    }

    /// Add member to room
    pub async fn add_member(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<RoomMember, AppError> {
        let member = sqlx::query_as::<_, RoomMember>(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, $3::member_role)
            RETURNING id, room_id, user_id, role::text as role, joined_at
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(pool)
        .await?;

        Ok(member)
    }

    /// Remove member from room
    pub async fn remove_member(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM room_members
            WHERE room_id = $1 AND user_id = $2
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotMember);
        }

        Ok(())
    }

    /// Get room members with user info
    pub async fn get_members(
        pool: &PgPool,
        room_id: Uuid,
    ) -> Result<Vec<RoomMemberResponse>, AppError> {
        let members = sqlx::query_as::<_, RoomMemberResponse>(
            r#"
            SELECT
                rm.id,
                rm.room_id,
                rm.user_id,
                u.username,
                u.display_name,
                u.avatar_url,
                rm.role::text as role,
                u.status,
                rm.joined_at
            FROM room_members rm
            JOIN users u ON rm.user_id = u.id
            WHERE rm.room_id = $1
            ORDER BY rm.joined_at ASC
            "#,
        )
        .bind(room_id)
        .fetch_all(pool)
        .await?;

        Ok(members)
    }

    /// Count room members
    pub async fn count_members(pool: &PgPool, room_id: Uuid) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM room_members WHERE room_id = $1
            "#,
        )
        .bind(room_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Check if user is member of room
    pub async fn is_member(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM room_members
                WHERE room_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }

    /// Get user's role in room
    pub async fn get_user_role(
        pool: &PgPool,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let role = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT role::text FROM room_members
            WHERE room_id = $1 AND user_id = $2
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(role.flatten())
    }

    /// Check if room name already exists
    pub async fn name_exists(pool: &PgPool, name: &str) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM rooms WHERE LOWER(name) = LOWER($1))
            "#,
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }
}
