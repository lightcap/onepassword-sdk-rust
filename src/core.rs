use std::collections::HashMap;

use serde::Serialize;

use crate::errors::SdkError;

pub(crate) const SDK_LANGUAGE: &str = "Rust";
pub(crate) const DEFAULT_REQUEST_LIBRARY: &str = "reqwest";
pub(crate) const MESSAGE_LIMIT: usize = 50 * 1024 * 1024;

pub(crate) trait Core: Send + Sync {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn release_client(&self, client_id: &[u8]);
}

#[derive(Debug, Serialize)]
pub(crate) struct ClientConfig {
    #[serde(rename = "serviceAccountToken")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_name: Option<String>,
}

impl ClientConfig {
    pub fn new_default() -> Self {
        Self {
            sa_token: String::new(),
            language: SDK_LANGUAGE.to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
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
        Ok(String::from_utf8_lossy(&res).into_owned())
    }

    pub fn release_client(&self, client_id: u64) {
        if let Ok(id_bytes) = serde_json::to_vec(&client_id) {
            self.inner.release_client(&id_bytes);
        }
    }
}

/// The inner client state shared by all API implementations.
pub(crate) struct InnerClient {
    pub id: u64,
    #[allow(dead_code)]
    pub config: ClientConfig,
    pub core: CoreWrapper,
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
}
