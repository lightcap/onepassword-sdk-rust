use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::types::*;

pub trait ItemsSharesApi {
    fn get_account_policy(
        &self,
        vault_id: &str,
        item_id: &str,
    ) -> Result<ItemShareAccountPolicy, SdkError>;
    fn validate_recipients(
        &self,
        policy: ItemShareAccountPolicy,
        recipients: &[String],
    ) -> Result<Vec<ValidRecipient>, SdkError>;
    fn create(
        &self,
        item: Item,
        policy: ItemShareAccountPolicy,
        params: ItemShareParams,
    ) -> Result<String, SdkError>;
}

pub(crate) struct ItemsSharesSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> ItemsSharesSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl ItemsSharesApi for ItemsSharesSource<'_> {
    fn get_account_policy(
        &self,
        vault_id: &str,
        item_id: &str,
    ) -> Result<ItemShareAccountPolicy, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "item_id".to_string(),
            serde_json::Value::String(item_id.to_string()),
        );
        let result = client_invoke(self.inner, "ItemsSharesGetAccountPolicy", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn validate_recipients(
        &self,
        policy: ItemShareAccountPolicy,
        recipients: &[String],
    ) -> Result<Vec<ValidRecipient>, SdkError> {
        let mut p = HashMap::new();
        p.insert("policy".to_string(), serde_json::to_value(&policy)?);
        p.insert("recipients".to_string(), serde_json::to_value(recipients)?);
        let result = client_invoke(self.inner, "ItemsSharesValidateRecipients", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn create(
        &self,
        item: Item,
        policy: ItemShareAccountPolicy,
        params: ItemShareParams,
    ) -> Result<String, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        p.insert("policy".to_string(), serde_json::to_value(&policy)?);
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "ItemsSharesCreate", p)?;
        Ok(serde_json::from_str(&result)?)
    }
}
