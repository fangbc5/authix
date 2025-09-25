use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthixError {
    #[error("JWT error: {0}")]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),

    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Unknown login type: {0}")]
    UnknowLoginType(String),

    #[error("Unknown register type: {0}")]
    UnknowRegisterType(String),

    #[error("Invalid credentials for {0}")]
    InvalidCredentials(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Redis error for {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Redis DeadPool error for {0}")]
    DeadPoolError(#[from] deadpool_redis::redis::RedisError)
}

pub type AuthixResult<T> = Result<T, AuthixError>;