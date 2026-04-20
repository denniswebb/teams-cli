use std::process;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TeamsError {
    #[error("authentication failed: {0}")]
    AuthError(String),

    #[error("token expired")]
    TokenExpired,

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("rate limited (retry after {retry_after_secs}s)")]
    RateLimited { retry_after_secs: u64 },

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("server error ({status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("config error: {0}")]
    ConfigError(String),

    #[error("keyring error: {0}")]
    KeyringError(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl TeamsError {
    pub fn exit_code(&self) -> i32 {
        match self {
            TeamsError::AuthError(_) | TeamsError::TokenExpired => 3,
            TeamsError::PermissionDenied(_) => 4,
            TeamsError::NotFound(_) => 5,
            TeamsError::RateLimited { .. } => 6,
            TeamsError::NetworkError(_) => 7,
            TeamsError::ServerError { .. } => 8,
            TeamsError::InvalidInput(_) => 2,
            TeamsError::ConfigError(_) | TeamsError::KeyringError(_) => 10,
            TeamsError::ApiError { status, .. } => match *status {
                401 => 3,
                403 => 4,
                404 => 5,
                429 => 6,
                s if s >= 500 => 8,
                _ => 1,
            },
            TeamsError::Other(_) => 1,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            TeamsError::AuthError(_) => "AUTH_FAILED",
            TeamsError::TokenExpired => "AUTH_TOKEN_EXPIRED",
            TeamsError::PermissionDenied(_) => "PERMISSION_DENIED",
            TeamsError::NotFound(_) => "NOT_FOUND",
            TeamsError::RateLimited { .. } => "RATE_LIMITED",
            TeamsError::NetworkError(_) => "NETWORK_ERROR",
            TeamsError::ServerError { .. } => "SERVER_ERROR",
            TeamsError::InvalidInput(_) => "INVALID_INPUT",
            TeamsError::ConfigError(_) => "CONFIG_ERROR",
            TeamsError::KeyringError(_) => "KEYRING_ERROR",
            TeamsError::ApiError { .. } => "API_ERROR",
            TeamsError::Other(_) => "UNKNOWN",
        }
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            TeamsError::AuthError(_)
                | TeamsError::TokenExpired
                | TeamsError::ApiError { status: 401, .. }
        )
    }

    #[allow(dead_code)]
    pub fn exit(&self) -> ! {
        tracing::error!("{self}");
        process::exit(self.exit_code());
    }
}

pub type Result<T> = std::result::Result<T, TeamsError>;
