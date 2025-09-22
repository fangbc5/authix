use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthixError {
    #[error("JWT error: {0}")]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),

    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Unknown login type: {0}")]
    UnknowLoginType(String),

    #[error("Invalid credentials for {0}")]
    InvalidCredentials(String),
}

pub type AuthixResult<T> = Result<T, AuthixError>;