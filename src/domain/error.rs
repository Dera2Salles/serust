
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid_credentials")]
    InvalidCredentials,

    #[error("user_not_found")]
    UserNotFound,

    #[error("file_not_found")]
    FileNotFound,

    #[error("permission_denied")]
    PermissionDenied,

    #[error("unsafe_path")]
    UnsafePath,

    #[error("file_too_large")]
    FileTooLarge,

    #[error("io_error: {0}")]
    Io(#[from] std::io::Error),

    #[error("internal_error: {0}")]
    Internal(String),
}
