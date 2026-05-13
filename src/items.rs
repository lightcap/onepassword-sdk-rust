use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::items_files::{ItemsFilesApi, ItemsFilesSource};
use crate::items_shares::{ItemsSharesApi, ItemsSharesSource};
use crate::types::*;

pub trait ItemsApi {
    fn create(&self, params: ItemCreateParams) -> Result<Item, SdkError>;
    fn create_all(
        &self,
        vault_id: &str,
        params: &[ItemCreateParams],
    ) -> Result<ItemsUpdateAllResponse, SdkError>;
    fn get(&self, vault_id: &str, item_id: &str) -> Result<Item, SdkError>;
    fn get_all(&self, vault_id: &str, item_ids: &[String])
    -> Result<ItemsGetAllResponse, SdkError>;
    fn put(&self, item: Item) -> Result<Item, SdkError>;
    fn delete(&self, vault_id: &str, item_id: &str) -> Result<(), SdkError>;
    fn delete_all(
        &self,
        vault_id: &str,
        item_ids: &[String],
    ) -> Result<ItemsDeleteAllResponse, SdkError>;
    fn archive(&self, vault_id: &str, item_id: &str) -> Result<(), SdkError>;
    fn list(
        &self,
        vault_id: &str,
        filters: &[ItemListFilter],
    ) -> Result<Vec<ItemOverview>, SdkError>;
    fn shares(&self) -> impl ItemsSharesApi + '_;
    fn files(&self) -> impl ItemsFilesApi + '_;
}

pub(crate) struct ItemsSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> ItemsSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl ItemsApi for ItemsSource<'_> {
    fn create(&self, params: ItemCreateParams) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "ItemsCreate", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn create_all(
        &self,
        vault_id: &str,
        params: &[ItemCreateParams],
    ) -> Result<ItemsUpdateAllResponse, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("params".to_string(), serde_json::to_value(params)?);
        let result = client_invoke(self.inner, "ItemsCreateAll", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn get(&self, vault_id: &str, item_id: &str) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "item_id".to_string(),
            serde_json::Value::String(item_id.to_string()),
        );
        let result = client_invoke(self.inner, "ItemsGet", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn get_all(
        &self,
        vault_id: &str,
        item_ids: &[String],
    ) -> Result<ItemsGetAllResponse, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("item_ids".to_string(), serde_json::to_value(item_ids)?);
        let result = client_invoke(self.inner, "ItemsGetAll", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn put(&self, item: Item) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        let result = client_invoke(self.inner, "ItemsPut", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn delete(&self, vault_id: &str, item_id: &str) -> Result<(), SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "item_id".to_string(),
            serde_json::Value::String(item_id.to_string()),
        );
        client_invoke(self.inner, "ItemsDelete", p)?;
        Ok(())
    }

    fn delete_all(
        &self,
        vault_id: &str,
        item_ids: &[String],
    ) -> Result<ItemsDeleteAllResponse, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("item_ids".to_string(), serde_json::to_value(item_ids)?);
        let result = client_invoke(self.inner, "ItemsDeleteAll", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn archive(&self, vault_id: &str, item_id: &str) -> Result<(), SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "item_id".to_string(),
            serde_json::Value::String(item_id.to_string()),
        );
        client_invoke(self.inner, "ItemsArchive", p)?;
        Ok(())
    }

    fn list(
        &self,
        vault_id: &str,
        filters: &[ItemListFilter],
    ) -> Result<Vec<ItemOverview>, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("filters".to_string(), serde_json::to_value(filters)?);
        let result = client_invoke(self.inner, "ItemsList", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn shares(&self) -> impl ItemsSharesApi + '_ {
        ItemsSharesSource::new(self.inner)
    }

    fn files(&self) -> impl ItemsFilesApi + '_ {
        ItemsFilesSource::new(self.inner)
    }
}
