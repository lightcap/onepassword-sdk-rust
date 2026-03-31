use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::types::*;

pub trait GroupsApi {
    fn get(&self, group_id: &str, params: GroupGetParams) -> Result<Group, SdkError>;
}

pub(crate) struct GroupsSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> GroupsSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl GroupsApi for GroupsSource<'_> {
    fn get(&self, group_id: &str, params: GroupGetParams) -> Result<Group, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "group_id".to_string(),
            serde_json::Value::String(group_id.to_string()),
        );
        p.insert("group_params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "GroupsGet", p)?;
        Ok(serde_json::from_str(&result)?)
    }
}
