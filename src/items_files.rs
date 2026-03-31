use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::types::*;

pub trait ItemsFilesApi {
    fn attach(&self, item: Item, file_params: FileCreateParams) -> Result<Item, SdkError>;
    fn read(
        &self,
        vault_id: &str,
        item_id: &str,
        attr: FileAttributes,
    ) -> Result<Vec<u8>, SdkError>;
    fn delete(&self, item: Item, section_id: &str, field_id: &str) -> Result<Item, SdkError>;
    fn replace_document(
        &self,
        item: Item,
        doc_params: DocumentCreateParams,
    ) -> Result<Item, SdkError>;
}

pub(crate) struct ItemsFilesSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> ItemsFilesSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl ItemsFilesApi for ItemsFilesSource<'_> {
    fn attach(&self, item: Item, file_params: FileCreateParams) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        p.insert(
            "file_params".to_string(),
            serde_json::to_value(&file_params)?,
        );
        let result = client_invoke(self.inner, "ItemsFilesAttach", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn read(
        &self,
        vault_id: &str,
        item_id: &str,
        attr: FileAttributes,
    ) -> Result<Vec<u8>, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "item_id".to_string(),
            serde_json::Value::String(item_id.to_string()),
        );
        p.insert("attr".to_string(), serde_json::to_value(&attr)?);
        let result = client_invoke(self.inner, "ItemsFilesRead", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn delete(&self, item: Item, section_id: &str, field_id: &str) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        p.insert(
            "section_id".to_string(),
            serde_json::Value::String(section_id.to_string()),
        );
        p.insert(
            "field_id".to_string(),
            serde_json::Value::String(field_id.to_string()),
        );
        let result = client_invoke(self.inner, "ItemsFilesDelete", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn replace_document(
        &self,
        item: Item,
        doc_params: DocumentCreateParams,
    ) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        p.insert("doc_params".to_string(), serde_json::to_value(&doc_params)?);
        let result = client_invoke(self.inner, "ItemsFilesReplaceDocument", p)?;
        Ok(serde_json::from_str(&result)?)
    }
}
