use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError{
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    IoError(String),

    #[error("internal error")]
    InternalError,
}

#[derive(Debug, Serialize)]
pub struct ErrorDto{
    pub code: String,
    pub message: String,
}

impl From<AppError> for ErrorDto{
    fn from(err: AppError) -> Self {
        let(code, message) = match err {
            AppError::InvalidArgument(msg) => ("INVALID_ARGUMENT", msg),
            AppError::NotFound(msg) => ("NOT_FOUND", msg),
            AppError::IoError(msg) => ("IO_ERROR", msg),
            AppError::InternalError => ("INTERNAL_ERROR", "An internal error occurred".to_string()),
        };

        Self {
            code: code.to_string(),
            message,
        }
    }
}