use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::types::GetVariablesResponse;

pub trait EnvironmentsApi {
    fn get_variables(&self, environment_id: &str) -> Result<GetVariablesResponse, SdkError>;
}

pub(crate) struct EnvironmentsSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> EnvironmentsSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl EnvironmentsApi for EnvironmentsSource<'_> {
    fn get_variables(&self, environment_id: &str) -> Result<GetVariablesResponse, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "environment_id".to_string(),
            serde_json::Value::String(environment_id.to_string()),
        );
        let result = client_invoke(self.inner, "EnvironmentsGetVariables", p)?;
        Ok(serde_json::from_str(&result)?)
    }
}
