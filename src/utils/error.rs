use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Databasfel: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO-fel: {0}")]
    Io(#[from] std::io::Error),

    #[error("Valideringsfel: {0}")]
    Validation(String),

    #[error("Hittades inte: {0}")]
    NotFound(String),

    #[error("Redan finns: {0}")]
    AlreadyExists(String),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

pub type AppResult<T> = Result<T, AppError>;
