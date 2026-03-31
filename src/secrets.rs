use std::collections::HashMap;
use std::sync::LazyLock;

use crate::client::client_invoke;
use crate::core::{CoreWrapper, InnerClient};
use crate::errors::SdkError;
use crate::types::{GeneratePasswordResponse, PasswordRecipe, ResolveAllResponse};

/// Shared ExtismCore instance for standalone operations (no client required).
/// Avoids recompiling the ~9.5MB WASM binary on every call.
static STANDALONE_CORE: LazyLock<Result<CoreWrapper, String>> = LazyLock::new(|| {
    let core = crate::core_extism::ExtismCore::new().map_err(|e| e.to_string())?;
    Ok(CoreWrapper {
        inner: Box::new(core),
    })
});

fn standalone_core() -> Result<&'static CoreWrapper, SdkError> {
    STANDALONE_CORE
        .as_ref()
        .map_err(|e| SdkError::Plugin(e.clone()))
}

pub trait SecretsApi {
    fn resolve(&self, secret_reference: &str) -> Result<String, SdkError>;
    fn resolve_all(&self, secret_references: &[String]) -> Result<ResolveAllResponse, SdkError>;
}

pub(crate) struct SecretsSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> SecretsSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl SecretsApi for SecretsSource<'_> {
    fn resolve(&self, secret_reference: &str) -> Result<String, SdkError> {
        let mut params = HashMap::new();
        params.insert(
            "secret_reference".to_string(),
            serde_json::Value::String(secret_reference.to_string()),
        );
        let result_string = client_invoke(self.inner, "SecretsResolve", params)?;
        let result: String = serde_json::from_str(&result_string)?;
        Ok(result)
    }

    fn resolve_all(&self, secret_references: &[String]) -> Result<ResolveAllResponse, SdkError> {
        let mut params = HashMap::new();
        params.insert(
            "secret_references".to_string(),
            serde_json::to_value(secret_references)?,
        );
        let result_string = client_invoke(self.inner, "SecretsResolveAll", params)?;
        let result: ResolveAllResponse = serde_json::from_str(&result_string)?;
        Ok(result)
    }
}

/// Standalone secret utilities (no client required).
pub struct Secrets;

impl Secrets {
    pub fn validate_secret_reference(secret_reference: &str) -> Result<(), SdkError> {
        let core = standalone_core()?;
        let invoke_config = crate::core::InvokeConfig {
            invocation: crate::core::Invocation {
                client_id: None,
                parameters: crate::core::Parameters {
                    name: "ValidateSecretReference".to_string(),
                    parameters: {
                        let mut m = HashMap::new();
                        m.insert(
                            "secret_reference".to_string(),
                            serde_json::Value::String(secret_reference.to_string()),
                        );
                        m
                    },
                },
            },
        };
        core.invoke(&invoke_config)?;
        Ok(())
    }

    pub fn generate_password(recipe: PasswordRecipe) -> Result<GeneratePasswordResponse, SdkError> {
        let core = standalone_core()?;
        let invoke_config = crate::core::InvokeConfig {
            invocation: crate::core::Invocation {
                client_id: None,
                parameters: crate::core::Parameters {
                    name: "GeneratePassword".to_string(),
                    parameters: {
                        let mut m = HashMap::new();
                        m.insert("recipe".to_string(), serde_json::to_value(&recipe)?);
                        m
                    },
                },
            },
        };
        let result_string = core.invoke(&invoke_config)?;
        let result: GeneratePasswordResponse = serde_json::from_str(&result_string)?;
        Ok(result)
    }
}
