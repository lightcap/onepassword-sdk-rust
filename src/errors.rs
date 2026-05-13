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

fn try_unmarshal_error(err: &str) -> Option<SdkError> {
    match serde_json::from_str::<CoreError>(err) {
        Ok(core_err) => match core_err.name.as_str() {
            "DesktopSessionExpired" => Some(SdkError::DesktopSessionExpired(core_err.message)),
            "RateLimitExceeded" => Some(SdkError::RateLimitExceeded(core_err.message)),
            _ => Some(SdkError::Core {
                name: core_err.name,
                message: core_err.message,
            }),
        },
        Err(_) => None,
    }
}

/// Deserialize a JSON error string from the WASM core into an SdkError.
pub(crate) fn unmarshal_error(err: &str) -> SdkError {
    try_unmarshal_error(err).unwrap_or_else(|| SdkError::Core {
        name: "Unknown".to_string(),
        message: err.to_string(),
    })
}

/// Convert backend transport errors into typed SDK errors when they contain core JSON.
pub(crate) fn unmarshal_core_error(err: SdkError) -> SdkError {
    match err {
        SdkError::Plugin(msg) => unmarshal_error(&msg),
        SdkError::SharedLib(msg) => try_unmarshal_error(&msg).unwrap_or(SdkError::SharedLib(msg)),
        err => err,
    }
}

pub(crate) fn invalid_utf8_error(err: std::string::FromUtf8Error) -> SdkError {
    SdkError::Core {
        name: "InvalidUtf8".to_string(),
        message: format!("core response was not valid UTF-8: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarshal_core_json_error() {
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

    #[test]
    fn unmarshal_core_error_preserves_non_json_shared_lib_error() {
        let err = unmarshal_core_error(SdkError::SharedLib("connection dropped".to_string()));
        match err {
            SdkError::SharedLib(message) => assert_eq!(message, "connection dropped"),
            _ => panic!("expected SharedLib error"),
        }
    }

    #[test]
    fn unmarshal_core_error_maps_json_shared_lib_error() {
        let err = unmarshal_core_error(SdkError::SharedLib(
            r#"{"name":"RateLimitExceeded","message":"slow down"}"#.to_string(),
        ));
        assert!(matches!(err, SdkError::RateLimitExceeded(_)));
    }
}
