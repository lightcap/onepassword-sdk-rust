use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("1Password SDK error: {message}")]
    Core { name: String, message: String },

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("WASM plugin error: {0}")]
    Plugin(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("desktop session expired: {0}")]
    DesktopSessionExpired(String),

    #[error("rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("shared library error: {0}")]
    SharedLib(String),
}

#[derive(Deserialize)]
struct CoreError {
    name: String,
    message: String,
}

/// Deserialize a JSON error string from the WASM core into an SdkError.
pub(crate) fn unmarshal_error(err: &str) -> SdkError {
    match serde_json::from_str::<CoreError>(err) {
        Ok(core_err) => match core_err.name.as_str() {
            "DesktopSessionExpired" => SdkError::DesktopSessionExpired(core_err.message),
            "RateLimitExceeded" => SdkError::RateLimitExceeded(core_err.message),
            _ => SdkError::Core {
                name: core_err.name,
                message: core_err.message,
            },
        },
        Err(_) => SdkError::Core {
            name: "Unknown".to_string(),
            message: err.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarshal_core_error() {
        let err = unmarshal_error(r#"{"name":"SomeError","message":"something broke"}"#);
        match err {
            SdkError::Core { name, message } => {
                assert_eq!(name, "SomeError");
                assert_eq!(message, "something broke");
            }
            _ => panic!("expected Core error"),
        }
    }

    #[test]
    fn unmarshal_session_expired() {
        let err = unmarshal_error(r#"{"name":"DesktopSessionExpired","message":"session gone"}"#);
        assert!(matches!(err, SdkError::DesktopSessionExpired(_)));
    }

    #[test]
    fn unmarshal_rate_limit() {
        let err = unmarshal_error(r#"{"name":"RateLimitExceeded","message":"slow down"}"#);
        assert!(matches!(err, SdkError::RateLimitExceeded(_)));
    }

    #[test]
    fn unmarshal_invalid_json() {
        let err = unmarshal_error("not json at all");
        match err {
            SdkError::Core { message, .. } => assert_eq!(message, "not json at all"),
            _ => panic!("expected Core error for invalid JSON"),
        }
    }
}
