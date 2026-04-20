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

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- exit_code() ---

    #[test]
    fn exit_code_auth_error() {
        let e = TeamsError::AuthError("bad creds".into());
        assert_eq!(e.exit_code(), 3);
    }

    #[test]
    fn exit_code_token_expired() {
        let e = TeamsError::TokenExpired;
        assert_eq!(e.exit_code(), 3);
    }

    #[test]
    fn exit_code_permission_denied() {
        let e = TeamsError::PermissionDenied("nope".into());
        assert_eq!(e.exit_code(), 4);
    }

    #[test]
    fn exit_code_not_found() {
        let e = TeamsError::NotFound("gone".into());
        assert_eq!(e.exit_code(), 5);
    }

    #[test]
    fn exit_code_rate_limited() {
        let e = TeamsError::RateLimited {
            retry_after_secs: 60,
        };
        assert_eq!(e.exit_code(), 6);
    }

    #[test]
    fn exit_code_server_error() {
        let e = TeamsError::ServerError {
            status: 502,
            message: "bad gateway".into(),
        };
        assert_eq!(e.exit_code(), 8);
    }

    #[test]
    fn exit_code_invalid_input() {
        let e = TeamsError::InvalidInput("bad".into());
        assert_eq!(e.exit_code(), 2);
    }

    #[test]
    fn exit_code_config_error() {
        let e = TeamsError::ConfigError("broken".into());
        assert_eq!(e.exit_code(), 10);
    }

    #[test]
    fn exit_code_keyring_error() {
        let e = TeamsError::KeyringError("locked".into());
        assert_eq!(e.exit_code(), 10);
    }

    #[test]
    fn exit_code_api_error_401() {
        let e = TeamsError::ApiError {
            status: 401,
            message: "unauthorized".into(),
        };
        assert_eq!(e.exit_code(), 3);
    }

    #[test]
    fn exit_code_api_error_403() {
        let e = TeamsError::ApiError {
            status: 403,
            message: "forbidden".into(),
        };
        assert_eq!(e.exit_code(), 4);
    }

    #[test]
    fn exit_code_api_error_404() {
        let e = TeamsError::ApiError {
            status: 404,
            message: "not found".into(),
        };
        assert_eq!(e.exit_code(), 5);
    }

    #[test]
    fn exit_code_api_error_429() {
        let e = TeamsError::ApiError {
            status: 429,
            message: "too many".into(),
        };
        assert_eq!(e.exit_code(), 6);
    }

    #[test]
    fn exit_code_api_error_500() {
        let e = TeamsError::ApiError {
            status: 500,
            message: "server error".into(),
        };
        assert_eq!(e.exit_code(), 8);
    }

    #[test]
    fn exit_code_api_error_503() {
        let e = TeamsError::ApiError {
            status: 503,
            message: "unavailable".into(),
        };
        assert_eq!(e.exit_code(), 8);
    }

    #[test]
    fn exit_code_api_error_418_catchall() {
        let e = TeamsError::ApiError {
            status: 418,
            message: "teapot".into(),
        };
        assert_eq!(e.exit_code(), 1);
    }

    #[test]
    fn exit_code_other() {
        let e = TeamsError::Other(anyhow::anyhow!("something"));
        assert_eq!(e.exit_code(), 1);
    }

    // --- error_code() ---

    #[test]
    fn error_code_auth_error() {
        assert_eq!(
            TeamsError::AuthError("x".into()).error_code(),
            "AUTH_FAILED"
        );
    }

    #[test]
    fn error_code_token_expired() {
        assert_eq!(TeamsError::TokenExpired.error_code(), "AUTH_TOKEN_EXPIRED");
    }

    #[test]
    fn error_code_permission_denied() {
        assert_eq!(
            TeamsError::PermissionDenied("x".into()).error_code(),
            "PERMISSION_DENIED"
        );
    }

    #[test]
    fn error_code_not_found() {
        assert_eq!(TeamsError::NotFound("x".into()).error_code(), "NOT_FOUND");
    }

    #[test]
    fn error_code_rate_limited() {
        assert_eq!(
            TeamsError::RateLimited {
                retry_after_secs: 1
            }
            .error_code(),
            "RATE_LIMITED"
        );
    }

    #[test]
    fn error_code_server_error() {
        assert_eq!(
            TeamsError::ServerError {
                status: 500,
                message: "x".into()
            }
            .error_code(),
            "SERVER_ERROR"
        );
    }

    #[test]
    fn error_code_invalid_input() {
        assert_eq!(
            TeamsError::InvalidInput("x".into()).error_code(),
            "INVALID_INPUT"
        );
    }

    #[test]
    fn error_code_config_error() {
        assert_eq!(
            TeamsError::ConfigError("x".into()).error_code(),
            "CONFIG_ERROR"
        );
    }

    #[test]
    fn error_code_keyring_error() {
        assert_eq!(
            TeamsError::KeyringError("x".into()).error_code(),
            "KEYRING_ERROR"
        );
    }

    #[test]
    fn error_code_api_error() {
        assert_eq!(
            TeamsError::ApiError {
                status: 401,
                message: "x".into()
            }
            .error_code(),
            "API_ERROR"
        );
    }

    #[test]
    fn error_code_other() {
        assert_eq!(
            TeamsError::Other(anyhow::anyhow!("x")).error_code(),
            "UNKNOWN"
        );
    }

    // --- is_auth_error() ---

    #[test]
    fn is_auth_error_auth_error() {
        assert!(TeamsError::AuthError("x".into()).is_auth_error());
    }

    #[test]
    fn is_auth_error_token_expired() {
        assert!(TeamsError::TokenExpired.is_auth_error());
    }

    #[test]
    fn is_auth_error_api_401() {
        assert!(TeamsError::ApiError {
            status: 401,
            message: "x".into()
        }
        .is_auth_error());
    }

    #[test]
    fn is_auth_error_api_403_is_false() {
        assert!(!TeamsError::ApiError {
            status: 403,
            message: "x".into()
        }
        .is_auth_error());
    }

    #[test]
    fn is_auth_error_permission_denied_is_false() {
        assert!(!TeamsError::PermissionDenied("x".into()).is_auth_error());
    }

    #[test]
    fn is_auth_error_other_is_false() {
        assert!(!TeamsError::Other(anyhow::anyhow!("x")).is_auth_error());
    }

    // --- Display trait ---

    #[test]
    fn display_auth_error() {
        let e = TeamsError::AuthError("bad token".into());
        assert!(e.to_string().contains("authentication failed"));
        assert!(e.to_string().contains("bad token"));
    }

    #[test]
    fn display_token_expired() {
        let e = TeamsError::TokenExpired;
        assert_eq!(e.to_string(), "token expired");
    }

    #[test]
    fn display_permission_denied() {
        let e = TeamsError::PermissionDenied("read scope".into());
        assert!(e.to_string().contains("permission denied"));
        assert!(e.to_string().contains("read scope"));
    }

    #[test]
    fn display_not_found() {
        let e = TeamsError::NotFound("chat 123".into());
        assert!(e.to_string().contains("not found"));
        assert!(e.to_string().contains("chat 123"));
    }

    #[test]
    fn display_rate_limited() {
        let e = TeamsError::RateLimited {
            retry_after_secs: 30,
        };
        let s = e.to_string();
        assert!(s.contains("rate limited"));
        assert!(s.contains("30"));
    }

    #[test]
    fn display_api_error() {
        let e = TeamsError::ApiError {
            status: 500,
            message: "boom".into(),
        };
        let s = e.to_string();
        assert!(s.contains("500"));
        assert!(s.contains("boom"));
    }

    #[test]
    fn display_server_error() {
        let e = TeamsError::ServerError {
            status: 502,
            message: "gateway".into(),
        };
        let s = e.to_string();
        assert!(s.contains("502"));
        assert!(s.contains("gateway"));
    }

    #[test]
    fn display_invalid_input() {
        let e = TeamsError::InvalidInput("missing field".into());
        assert!(e.to_string().contains("invalid input"));
        assert!(e.to_string().contains("missing field"));
    }

    #[test]
    fn display_config_error() {
        let e = TeamsError::ConfigError("parse fail".into());
        assert!(e.to_string().contains("config error"));
        assert!(e.to_string().contains("parse fail"));
    }

    #[test]
    fn display_keyring_error() {
        let e = TeamsError::KeyringError("locked".into());
        assert!(e.to_string().contains("keyring error"));
        assert!(e.to_string().contains("locked"));
    }

    #[test]
    fn display_other() {
        let e = TeamsError::Other(anyhow::anyhow!("mystery"));
        assert!(e.to_string().contains("mystery"));
    }
}
