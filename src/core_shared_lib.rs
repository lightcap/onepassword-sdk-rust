// Stub: SharedLibCore for desktop app integration (feature: desktop).
// Full implementation will be added in a later task.

use crate::errors::SdkError;

#[allow(dead_code)]
pub(crate) struct SharedLibCore;

impl SharedLibCore {
    #[allow(dead_code)]
    pub fn new(_account_name: &str) -> Result<Self, SdkError> {
        Err(SdkError::SharedLib(
            "SharedLibCore is not yet implemented".to_string(),
        ))
    }
}

impl crate::core::Core for SharedLibCore {
    fn init_client(&self, _config: &[u8]) -> Result<Vec<u8>, SdkError> {
        Err(SdkError::SharedLib("not implemented".to_string()))
    }

    fn invoke(&self, _invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
        Err(SdkError::SharedLib("not implemented".to_string()))
    }

    fn release_client(&self, _client_id: &[u8]) {}
}
