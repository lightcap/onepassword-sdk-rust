use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crate::core::{ClientConfig, CoreWrapper, InnerClient, Invocation, InvokeConfig, Parameters};
use crate::core_extism::ExtismCore;
use crate::environments::{EnvironmentsApi, EnvironmentsSource};
use crate::errors::{SdkError, unmarshal_core_error};
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
            .field("id", &self.inner.client_id())
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
        self.inner.core.release_client(self.inner.client_id());
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
            .map_err(unmarshal_core_error)?;

        let inner = Arc::new(InnerClient {
            id: AtomicU64::new(client_id),
            config: self.config,
            core,
            retry_lock: std::sync::Mutex::new(()),
        });

        Ok(Client { inner })
    }
}

/// Invoke a method on the WASM core. Used by all API implementations.
/// Automatically retries once on DesktopSessionExpired by re-initializing the client.
pub(crate) fn client_invoke(
    inner: &InnerClient,
    method: &str,
    params: HashMap<String, serde_json::Value>,
) -> Result<String, SdkError> {
    let invoke_config = InvokeConfig {
        invocation: Invocation {
            client_id: Some(inner.client_id()),
            parameters: Parameters {
                name: method.to_string(),
                parameters: params,
            },
        },
    };

    match inner.core.invoke(&invoke_config) {
        Ok(response) => Ok(response),
        Err(err) => {
            let err = unmarshal_core_error(err);
            if matches!(err, SdkError::DesktopSessionExpired(_)) {
                retry_invoke(
                    inner,
                    method,
                    invoke_config.invocation.parameters.parameters,
                )
            } else {
                Err(err)
            }
        }
    }
}

fn retry_invoke(
    inner: &InnerClient,
    method: &str,
    params: HashMap<String, serde_json::Value>,
) -> Result<String, SdkError> {
    let _guard = inner
        .retry_lock
        .lock()
        .map_err(|e| SdkError::Config(format!("retry lock poisoned: {e}")))?;
    let old_id = inner.client_id();
    let new_id = inner
        .core
        .init_client(&inner.config)
        .map_err(unmarshal_core_error)?;
    inner.core.release_client(old_id);
    inner.set_client_id(new_id);
    let retry_config = InvokeConfig {
        invocation: Invocation {
            client_id: Some(new_id),
            parameters: Parameters {
                name: method.to_string(),
                parameters: params,
            },
        },
    };
    inner
        .core
        .invoke(&retry_config)
        .map_err(unmarshal_core_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn builder_rejects_both_auth_methods() {
        let mut builder = Client::builder().service_account_token("ops_test");
        builder.config.account_name = Some("myaccount".to_string());
        let result = builder.build();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot use both"));
    }

    struct RetryCore {
        invoke_calls: AtomicUsize,
    }

    impl crate::core::Core for RetryCore {
        fn init_client(&self, _config: &[u8]) -> Result<Vec<u8>, SdkError> {
            Ok(serde_json::to_vec(&99u64).unwrap())
        }

        fn invoke(&self, _invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
            if self.invoke_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                Err(SdkError::SharedLib(
                    r#"{"name":"DesktopSessionExpired","message":"expired"}"#.to_string(),
                ))
            } else {
                Ok(br#""ok""#.to_vec())
            }
        }

        fn release_client(&self, _client_id: &[u8]) {}
    }

    #[test]
    fn client_invoke_retries_shared_lib_session_expired() {
        let inner = InnerClient {
            id: AtomicU64::new(1),
            config: ClientConfig::new_default(),
            core: CoreWrapper {
                inner: Box::new(RetryCore {
                    invoke_calls: AtomicUsize::new(0),
                }),
            },
            retry_lock: std::sync::Mutex::new(()),
        };

        let response = client_invoke(&inner, "SecretsResolve", HashMap::new()).unwrap();
        assert_eq!(response, r#""ok""#);
        assert_eq!(inner.client_id(), 99);
    }
}
