use thiserror::Error;

#[derive(Debug, Error)]
pub enum FreshBluError {
    #[error("device not found")]
    NotFound,

    #[error("forbidden")]
    Forbidden,

    #[error("unauthorized")]
    Unauthorized,

    #[error("invalid token")]
    InvalidToken,

    #[error("device already exists")]
    Conflict,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("rate limit exceeded")]
    RateLimitExceeded,

    #[error("message too large")]
    MessageTooLarge,
}

impl FreshBluError {
    pub fn http_status(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::Forbidden => 403,
            Self::Unauthorized => 401,
            Self::InvalidToken => 401,
            Self::Conflict => 409,
            Self::Validation(_) => 422,
            Self::RateLimitExceeded => 429,
            Self::MessageTooLarge => 413,
            _ => 500,
        }
    }
}

pub type Result<T> = std::result::Result<T, FreshBluError>;
