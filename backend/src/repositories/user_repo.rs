use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use crate::models::user::{User, CreateUserDto, UpdateUserDto};

pub struct UserRepository;

impl UserRepository {
    /// Create a new user
    pub async fn create(pool: &PgPool, dto: &CreateUserDto, password_hash: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, display_name, status)
            VALUES ($1, $2, $3, $4, 'offline')
            RETURNING *
            "#
        )
        .bind(&dto.username)
        .bind(&dto.email)
        .bind(password_hash)
        .bind(&dto.display_name)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE email = $1 AND is_active = true
            "#
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1 AND is_active = true
            "#
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by username
    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE username = $1 AND is_active = true
            "#
        )
        .bind(username)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Update user
    pub async fn update(pool: &PgPool, user_id: Uuid, dto: &UpdateUserDto) -> Result<User, AppError> {
        // Build dynamic update query based on provided fields
        let mut query = String::from("UPDATE users SET ");
        let mut updates = Vec::new();
        let mut param_count = 1;

        if let Some(_) = &dto.username {
            updates.push(format!("username = ${}", param_count));
            param_count += 1;
        }
        if let Some(_) = &dto.display_name {
            updates.push(format!("display_name = ${}", param_count));
            param_count += 1;
        }
        if let Some(_) = &dto.avatar_url {
            updates.push(format!("avatar_url = ${}", param_count));
            param_count += 1;
        }
        if let Some(_) = &dto.status {
            updates.push(format!("status = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return Self::find_by_id(pool, user_id).await;
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(", updated_at = NOW() WHERE id = ${} AND is_active = true RETURNING *", param_count));

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(username) = &dto.username {
            query_builder = query_builder.bind(username);
        }
        if let Some(display_name) = &dto.display_name {
            query_builder = query_builder.bind(display_name);
        }
        if let Some(avatar_url) = &dto.avatar_url {
            query_builder = query_builder.bind(avatar_url);
        }
        if let Some(status) = &dto.status {
            query_builder = query_builder.bind(status);
        }

        query_builder = query_builder.bind(user_id);

        let user = query_builder.fetch_one(pool).await?;

        Ok(user)
    }

    /// Update user status (online/offline/away/busy)
    pub async fn update_status(pool: &PgPool, user_id: Uuid, status: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users 
            SET status = $1, updated_at = NOW()
            WHERE id = $2 AND is_active = true
            "#
        )
        .bind(status)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, AppError> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)
            "#
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// Check if username exists
    pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, AppError> {
        let result: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)
            "#
        )
        .bind(username)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}
