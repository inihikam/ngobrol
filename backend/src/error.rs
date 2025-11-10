use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

/// Enterprise-grade error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
}

/// Validation errors with field-specific messages
#[derive(Debug, Serialize, Clone)]
pub struct ValidationErrors {
    #[serde(flatten)]
    pub fields: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field_error(&mut self, field: &str, message: &str) {
        self.fields
            .entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(message.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Main error enum for the application
#[derive(Debug)]
pub enum AppError {
    // Authentication errors (AUTH_*)
    MissingToken,
    InvalidToken,
    InvalidCredentials,
    TokenExpired,
    AccountLocked,
    InsufficientPermissions,

    // User errors (USER_*)
    UserNotFound,
    EmailExists,
    UsernameExists,
    InvalidEmail,
    WeakPassword,

    // Room errors (ROOM_*)
    RoomNotFound,
    AlreadyJoined,
    NotMember,
    RoomFull,
    RoomNameExists,
    PrivateNoAccess,
    OwnerRequired,

    // Message errors (MESSAGE_*)
    MessageNotFound,
    MessageEmpty,
    MessageTooLong,
    NotMessageOwner,
    MessageAlreadyDeleted,

    // Validation errors (VALIDATION_*)
    ValidationError(ValidationErrors),
    MissingField(String),
    InvalidFormat(String),
    InvalidUuid(String),

    // Rate limiting (RATE_LIMIT_*)
    RateLimitExceeded,
    MessageSpam,
    LoginAttempts,

    // Server errors (SERVER_*)
    DatabaseError(String),
    RedisError(String),
    InternalError(String),
}

impl AppError {
    /// Get standardized error code
    pub fn code(&self) -> &str {
        match self {
            // Auth errors
            Self::MissingToken => "AUTH_MISSING_TOKEN",
            Self::InvalidToken => "AUTH_INVALID_TOKEN",
            Self::InvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            Self::TokenExpired => "AUTH_TOKEN_EXPIRED",
            Self::AccountLocked => "AUTH_ACCOUNT_LOCKED",
            Self::InsufficientPermissions => "AUTH_INSUFFICIENT_PERMISSIONS",

            // User errors
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::EmailExists => "USER_EMAIL_EXISTS",
            Self::UsernameExists => "USER_USERNAME_EXISTS",
            Self::InvalidEmail => "USER_INVALID_EMAIL",
            Self::WeakPassword => "USER_WEAK_PASSWORD",

            // Room errors
            Self::RoomNotFound => "ROOM_NOT_FOUND",
            Self::AlreadyJoined => "ROOM_ALREADY_JOINED",
            Self::NotMember => "ROOM_NOT_MEMBER",
            Self::RoomFull => "ROOM_FULL",
            Self::RoomNameExists => "ROOM_NAME_EXISTS",
            Self::PrivateNoAccess => "ROOM_PRIVATE_NO_ACCESS",
            Self::OwnerRequired => "ROOM_OWNER_REQUIRED",

            // Message errors
            Self::MessageNotFound => "MESSAGE_NOT_FOUND",
            Self::MessageEmpty => "MESSAGE_EMPTY",
            Self::MessageTooLong => "MESSAGE_TOO_LONG",
            Self::NotMessageOwner => "MESSAGE_NOT_OWNER",
            Self::MessageAlreadyDeleted => "MESSAGE_ALREADY_DELETED",

            // Validation
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::MissingField(_) => "VALIDATION_MISSING_FIELD",
            Self::InvalidFormat(_) => "VALIDATION_INVALID_FORMAT",
            Self::InvalidUuid(_) => "VALIDATION_INVALID_UUID",

            // Rate limit
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::MessageSpam => "RATE_LIMIT_MESSAGE_SPAM",
            Self::LoginAttempts => "RATE_LIMIT_LOGIN_ATTEMPTS",

            // Server errors
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::RedisError(_) => "REDIS_ERROR",
            Self::InternalError(_) => "INTERNAL_SERVER_ERROR",
        }
    }

    /// Get user-friendly error message
    pub fn message(&self) -> String {
        match self {
            // Auth errors
            Self::MissingToken => "Authentication token is required",
            Self::InvalidToken => "Invalid or expired authentication token",
            Self::InvalidCredentials => "Invalid email or password",
            Self::TokenExpired => "Authentication token has expired",
            Self::AccountLocked => "Your account has been locked",
            Self::InsufficientPermissions => "You don't have permission to perform this action",

            // User errors
            Self::UserNotFound => "User not found",
            Self::EmailExists => "Email address is already registered",
            Self::UsernameExists => "Username is already taken",
            Self::InvalidEmail => "Invalid email format",
            Self::WeakPassword => "Password does not meet requirements",

            // Room errors
            Self::RoomNotFound => "Room not found",
            Self::AlreadyJoined => "You have already joined this room",
            Self::NotMember => "You are not a member of this room",
            Self::RoomFull => "Room has reached maximum capacity",
            Self::RoomNameExists => "Room name is already taken",
            Self::PrivateNoAccess => "This is a private room",
            Self::OwnerRequired => "Only room owner can perform this action",

            // Message errors
            Self::MessageNotFound => "Message not found",
            Self::MessageEmpty => "Message content cannot be empty",
            Self::MessageTooLong => "Message exceeds maximum length",
            Self::NotMessageOwner => "You can only edit/delete your own messages",
            Self::MessageAlreadyDeleted => "Message has already been deleted",

            // Validation
            Self::ValidationError(_) => "Input validation failed",
            Self::MissingField(field) => return format!("Required field '{}' is missing", field),
            Self::InvalidFormat(field) => return format!("Invalid format for field '{}'", field),
            Self::InvalidUuid(field) => return format!("Invalid UUID format for field '{}'", field),

            // Rate limit
            Self::RateLimitExceeded => "Too many requests. Please try again later",
            Self::MessageSpam => "You are sending messages too quickly",
            Self::LoginAttempts => "Too many login attempts. Please try again in 15 minutes",

            // Server errors
            Self::DatabaseError(_) => "Database operation failed",
            Self::RedisError(_) => "Cache service error",
            Self::InternalError(_) => "An unexpected error occurred. Please try again later",
        }
        .to_string()
    }

    /// Get HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 401 Unauthorized
            Self::MissingToken | Self::InvalidToken | Self::InvalidCredentials | Self::TokenExpired => {
                StatusCode::UNAUTHORIZED
            }

            // 403 Forbidden
            Self::AccountLocked
            | Self::InsufficientPermissions
            | Self::NotMember
            | Self::NotMessageOwner
            | Self::PrivateNoAccess
            | Self::OwnerRequired => StatusCode::FORBIDDEN,

            // 404 Not Found
            Self::UserNotFound | Self::RoomNotFound | Self::MessageNotFound => StatusCode::NOT_FOUND,

            // 409 Conflict
            Self::EmailExists
            | Self::UsernameExists
            | Self::AlreadyJoined
            | Self::RoomFull
            | Self::RoomNameExists
            | Self::MessageAlreadyDeleted => StatusCode::CONFLICT,

            // 422 Unprocessable Entity (for validation)
            Self::ValidationError(_)
            | Self::MissingField(_)
            | Self::InvalidFormat(_)
            | Self::InvalidUuid(_)
            | Self::InvalidEmail
            | Self::WeakPassword
            | Self::MessageEmpty
            | Self::MessageTooLong => StatusCode::UNPROCESSABLE_ENTITY,

            // 429 Too Many Requests
            Self::RateLimitExceeded | Self::MessageSpam | Self::LoginAttempts => {
                StatusCode::TOO_MANY_REQUESTS
            }

            // 500 Internal Server Error
            Self::DatabaseError(_) | Self::RedisError(_) | Self::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Convert to ErrorResponse with timestamp
    pub fn to_response(&self) -> ErrorResponse {
        let details = match self {
            Self::ValidationError(validation_errors) => {
                Some(serde_json::to_value(&validation_errors.fields).unwrap())
            }
            _ => None,
        };

        ErrorResponse {
            error: ErrorDetail {
                code: self.code().to_string(),
                message: self.message(),
                details,
                timestamp: Utc::now().to_rfc3339(),
            },
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let body = self.to_response();

        // Log error (don't expose internal details in production)
        match self {
            Self::DatabaseError(msg) | Self::RedisError(msg) | Self::InternalError(msg) => {
                log::error!("Internal error [{}]: {}", self.code(), msg);
            }
            _ => {
                log::warn!("Client error [{}]: {}", self.code(), self.message());
            }
        }

        HttpResponse::build(status).json(body)
    }

    fn status_code(&self) -> StatusCode {
        self.status_code()
    }
}

// Implement From trait for common error conversions
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::UserNotFound,
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violation (PostgreSQL error code 23505)
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        // Try to determine which field based on constraint name
                        let constraint = db_err.constraint().unwrap_or("");
                        if constraint.contains("email") {
                            return AppError::EmailExists;
                        } else if constraint.contains("username") {
                            return AppError::UsernameExists;
                        }
                        // Default duplicate error
                        return AppError::EmailExists;
                    }
                }
                log::error!("Database error: {:?}", db_err);
                AppError::DatabaseError("Database operation failed".to_string())
            }
            _ => {
                log::error!("Database error: {:?}", err);
                AppError::DatabaseError("Database operation failed".to_string())
            }
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        log::error!("Redis error: {:?}", err);
        AppError::RedisError("Cache operation failed".to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        log::error!("Password hashing error: {:?}", err);
        AppError::InternalError("Password hashing failed".to_string())
    }
}

/// Type alias for Result with AppError
pub type AppResult<T> = Result<T, AppError>;
