use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::errors::{SdkError, invalid_utf8_error};

pub(crate) const SDK_LANGUAGE: &str = "Rust";
pub(crate) const DEFAULT_REQUEST_LIBRARY: &str = "reqwest";
pub(crate) const MESSAGE_LIMIT: usize = 50 * 1024 * 1024;

pub(crate) trait Core: Send + Sync {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn release_client(&self, client_id: &[u8]);
}

#[derive(Serialize)]
pub(crate) struct ClientConfig {
    #[serde(
        rename = "serviceAccountToken",
        skip_serializing_if = "String::is_empty"
    )]
    pub sa_token: String,
    #[serde(rename = "programmingLanguage")]
    pub language: String,
    #[serde(rename = "sdkVersion")]
    pub sdk_version: String,
    #[serde(rename = "integrationName")]
    pub integration_name: String,
    #[serde(rename = "integrationVersion")]
    pub integration_version: String,
    #[serde(rename = "requestLibraryName")]
    pub request_library_name: String,
    #[serde(rename = "requestLibraryVersion")]
    pub request_library_version: String,
    #[serde(rename = "os")]
    pub system_os: String,
    #[serde(rename = "osVersion")]
    pub system_os_version: String,
    #[serde(rename = "architecture")]
    pub system_arch: String,
    #[serde(skip_serializing)]
    pub account_name: Option<String>,
}

impl std::fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConfig")
            .field("sa_token", &"[REDACTED]")
            .field("language", &self.language)
            .field("sdk_version", &self.sdk_version)
            .field("integration_name", &self.integration_name)
            .field("integration_version", &self.integration_version)
            .field("system_os", &self.system_os)
            .field("system_arch", &self.system_arch)
            .finish()
    }
}

impl ClientConfig {
    pub fn new_default() -> Self {
        Self {
            sa_token: String::new(),
            language: SDK_LANGUAGE.to_string(),
            // Must match the Go SDK's version-build format (7-digit semver: Mmmppbb)
            // for the WASM core we embedded (v0.4.1-beta.1 = 0040101)
            sdk_version: "0040101".to_string(),
            integration_name: "Unknown".to_string(),
            integration_version: "Unknown".to_string(),
            request_library_name: DEFAULT_REQUEST_LIBRARY.to_string(),
            request_library_version: "0.0.0".to_string(),
            system_os: std::env::consts::OS.to_string(),
            system_os_version: "0.0.0".to_string(),
            system_arch: std::env::consts::ARCH.to_string(),
            account_name: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct InvokeConfig {
    pub invocation: Invocation,
}

#[derive(Debug, Serialize)]
pub(crate) struct Invocation {
    #[serde(rename = "clientId", skip_serializing_if = "Option::is_none")]
    pub client_id: Option<u64>,
    pub parameters: Parameters,
}

#[derive(Debug, Serialize)]
pub(crate) struct Parameters {
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Wraps a Core implementation and handles JSON marshaling/unmarshaling.
pub(crate) struct CoreWrapper {
    pub inner: Box<dyn Core>,
}

impl CoreWrapper {
    pub fn init_client(&self, config: &ClientConfig) -> Result<u64, SdkError> {
        let config_bytes = serde_json::to_vec(config)?;
        let res = self.inner.init_client(&config_bytes)?;
        let id: u64 = serde_json::from_slice(&res)?;
        Ok(id)
    }

    pub fn invoke(&self, invoke_config: &InvokeConfig) -> Result<String, SdkError> {
        let input = serde_json::to_vec(invoke_config)?;
        if input.len() > MESSAGE_LIMIT {
            return Err(SdkError::Config(format!(
                "message size exceeds the limit of {} bytes",
                MESSAGE_LIMIT
            )));
        }
        let res = self.inner.invoke(&input)?;
        String::from_utf8(res).map_err(invalid_utf8_error)
    }

    pub fn release_client(&self, client_id: u64) {
        if let Ok(id_bytes) = serde_json::to_vec(&client_id) {
            self.inner.release_client(&id_bytes);
        }
    }
}

/// The inner client state shared by all API implementations.
pub(crate) struct InnerClient {
    pub id: AtomicU64,
    pub config: ClientConfig,
    pub core: CoreWrapper,
    pub retry_lock: Mutex<()>,
}

impl InnerClient {
    pub fn client_id(&self) -> u64 {
        self.id.load(Ordering::Acquire)
    }

    pub fn set_client_id(&self, id: u64) {
        self.id.store(id, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_language() {
        let config = ClientConfig::new_default();
        assert_eq!(config.language, "Rust");
        assert_eq!(config.system_os, std::env::consts::OS);
        assert_eq!(config.system_arch, std::env::consts::ARCH);
    }

    #[test]
    fn invoke_config_serializes_correctly() {
        let config = InvokeConfig {
            invocation: Invocation {
                client_id: Some(42),
                parameters: Parameters {
                    name: "SecretsResolve".to_string(),
                    parameters: {
                        let mut m = HashMap::new();
                        m.insert(
                            "secret_reference".to_string(),
                            serde_json::Value::String("op://vault/item/field".to_string()),
                        );
                        m
                    },
                },
            },
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"clientId\":42"));
        assert!(json.contains("\"name\":\"SecretsResolve\""));
    }

    #[test]
    fn service_account_config_serializes_token() {
        let mut config = ClientConfig::new_default();
        config.sa_token = "ops_test".to_string();

        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["serviceAccountToken"], "ops_test");
    }

    #[test]
    fn desktop_config_omits_auth_fields_from_core_config() {
        let mut config = ClientConfig::new_default();
        config.account_name = Some("myaccount".to_string());

        let value = serde_json::to_value(&config).unwrap();
        assert!(value.get("serviceAccountToken").is_none());
        assert!(value.get("account_name").is_none());
    }

    struct InvalidUtf8Core;

    impl Core for InvalidUtf8Core {
        fn init_client(&self, _config: &[u8]) -> Result<Vec<u8>, SdkError> {
            Ok(Vec::new())
        }

        fn invoke(&self, _invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
            Ok(vec![0xff])
        }

        fn release_client(&self, _client_id: &[u8]) {}
    }

    #[test]
    fn invoke_rejects_invalid_utf8_response() {
        let wrapper = CoreWrapper {
            inner: Box::new(InvalidUtf8Core),
        };
        let config = InvokeConfig {
            invocation: Invocation {
                client_id: None,
                parameters: Parameters {
                    name: "InvalidUtf8".to_string(),
                    parameters: HashMap::new(),
                },
            },
        };

        let err = wrapper.invoke(&config).unwrap_err();
        match err {
            SdkError::Core { name, message } => {
                assert_eq!(name, "InvalidUtf8");
                assert!(message.contains("not valid UTF-8"));
            }
            _ => panic!("expected InvalidUtf8 core error"),
        }
    }
}
