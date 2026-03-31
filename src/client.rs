use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{ClientConfig, CoreWrapper, InnerClient, Invocation, InvokeConfig, Parameters};
use crate::core_extism::ExtismCore;
use crate::environments::{EnvironmentsApi, EnvironmentsSource};
use crate::errors::{SdkError, unmarshal_error};
use crate::groups::{GroupsApi, GroupsSource};
use crate::items::{ItemsApi, ItemsSource};
use crate::secrets::{SecretsApi, SecretsSource};
use crate::vaults::{VaultsApi, VaultsSource};

pub struct Client {
    inner: Arc<InnerClient>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("id", &self.inner.id)
            .finish()
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            config: ClientConfig::new_default(),
        }
    }

    pub fn secrets(&self) -> impl SecretsApi + '_ {
        SecretsSource::new(&self.inner)
    }

    pub fn items(&self) -> impl ItemsApi + '_ {
        ItemsSource::new(&self.inner)
    }

    pub fn vaults(&self) -> impl VaultsApi + '_ {
        VaultsSource::new(&self.inner)
    }

    pub fn groups(&self) -> impl GroupsApi + '_ {
        GroupsSource::new(&self.inner)
    }

    pub fn environments(&self) -> impl EnvironmentsApi + '_ {
        EnvironmentsSource::new(&self.inner)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.inner.core.release_client(self.inner.id);
    }
}

pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn service_account_token(mut self, token: &str) -> Self {
        self.config.sa_token = token.to_string();
        self
    }

    #[cfg(feature = "desktop")]
    pub fn desktop_app_integration(mut self, account_name: &str) -> Self {
        self.config.account_name = Some(account_name.to_string());
        self
    }

    pub fn integration_info(mut self, name: &str, version: &str) -> Self {
        self.config.integration_name = name.to_string();
        self.config.integration_version = version.to_string();
        self
    }

    pub fn build(self) -> Result<Client, SdkError> {
        let has_sa_token = !self.config.sa_token.is_empty();
        let has_desktop = self.config.account_name.is_some();

        if has_sa_token && has_desktop {
            return Err(SdkError::Config(
                "cannot use both SA token and desktop app authentication".to_string(),
            ));
        }

        if !has_sa_token && !has_desktop {
            return Err(SdkError::Config(
                "must set either service_account_token or desktop_app_integration".to_string(),
            ));
        }

        let core_impl: Box<dyn crate::core::Core> = if has_desktop {
            #[cfg(feature = "desktop")]
            {
                let account_name = self.config.account_name.as_ref().unwrap().clone();
                Box::new(crate::core_shared_lib::SharedLibCore::new(&account_name)?)
            }
            #[cfg(not(feature = "desktop"))]
            {
                return Err(SdkError::Config(
                    "desktop app integration requires the 'desktop' feature".to_string(),
                ));
            }
        } else {
            Box::new(ExtismCore::new()?)
        };

        let core = CoreWrapper { inner: core_impl };

        let client_id = core
            .init_client(&self.config)
            .map_err(|e| SdkError::Config(format!("error initializing client: {e}")))?;

        let inner = Arc::new(InnerClient {
            id: client_id,
            config: self.config,
            core,
        });

        Ok(Client { inner })
    }
}

/// Invoke a method on the WASM core. Used by all API implementations.
pub(crate) fn client_invoke(
    inner: &InnerClient,
    method: &str,
    params: HashMap<String, serde_json::Value>,
) -> Result<String, SdkError> {
    let invoke_config = InvokeConfig {
        invocation: Invocation {
            client_id: Some(inner.id),
            parameters: Parameters {
                name: method.to_string(),
                parameters: params,
            },
        },
    };

    match inner.core.invoke(&invoke_config) {
        Ok(response) => Ok(response),
        Err(SdkError::Plugin(msg)) => {
            // Extism surfaces WASM core errors as plugin errors containing JSON.
            // Try to parse the JSON error; if it fails, return the plugin error as-is.
            Err(unmarshal_error(&msg))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_rejects_both_auth_methods() {
        let mut builder = Client::builder().service_account_token("ops_test");
        builder.config.account_name = Some("myaccount".to_string());
        let result = builder.build();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot use both"));
    }
}
