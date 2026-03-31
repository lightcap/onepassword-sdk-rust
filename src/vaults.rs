use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::types::*;

pub trait VaultsApi {
    fn create(&self, params: VaultCreateParams) -> Result<Vault, SdkError>;
    fn list(&self, params: Option<VaultListParams>) -> Result<Vec<VaultOverview>, SdkError>;
    fn get_overview(&self, vault_id: &str) -> Result<VaultOverview, SdkError>;
    fn get(&self, vault_id: &str, params: VaultGetParams) -> Result<Vault, SdkError>;
    fn update(&self, vault_id: &str, params: VaultUpdateParams) -> Result<Vault, SdkError>;
    fn delete(&self, vault_id: &str) -> Result<(), SdkError>;
    fn grant_group_permissions(
        &self,
        vault_id: &str,
        group_permissions: &[GroupAccess],
    ) -> Result<(), SdkError>;
    fn update_group_permissions(
        &self,
        group_permissions: &[GroupVaultAccess],
    ) -> Result<(), SdkError>;
    fn revoke_group_permissions(&self, vault_id: &str, group_id: &str) -> Result<(), SdkError>;
}

pub(crate) struct VaultsSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> VaultsSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl VaultsApi for VaultsSource<'_> {
    fn create(&self, params: VaultCreateParams) -> Result<Vault, SdkError> {
        let mut p = HashMap::new();
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "VaultsCreate", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn list(&self, params: Option<VaultListParams>) -> Result<Vec<VaultOverview>, SdkError> {
        let mut p = HashMap::new();
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "VaultsList", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn get_overview(&self, vault_id: &str) -> Result<VaultOverview, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        let result = client_invoke(self.inner, "VaultsGetOverview", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn get(&self, vault_id: &str, params: VaultGetParams) -> Result<Vault, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("vault_params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "VaultsGet", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn update(&self, vault_id: &str, params: VaultUpdateParams) -> Result<Vault, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result = client_invoke(self.inner, "VaultsUpdate", p)?;
        Ok(serde_json::from_str(&result)?)
    }

    fn delete(&self, vault_id: &str) -> Result<(), SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        client_invoke(self.inner, "VaultsDelete", p)?;
        Ok(())
    }

    fn grant_group_permissions(
        &self,
        vault_id: &str,
        group_permissions: &[GroupAccess],
    ) -> Result<(), SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "group_permissions_list".to_string(),
            serde_json::to_value(group_permissions)?,
        );
        client_invoke(self.inner, "VaultsGrantGroupPermissions", p)?;
        Ok(())
    }

    fn update_group_permissions(
        &self,
        group_permissions: &[GroupVaultAccess],
    ) -> Result<(), SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "group_permissions_list".to_string(),
            serde_json::to_value(group_permissions)?,
        );
        client_invoke(self.inner, "VaultsUpdateGroupPermissions", p)?;
        Ok(())
    }

    fn revoke_group_permissions(&self, vault_id: &str, group_id: &str) -> Result<(), SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert(
            "group_id".to_string(),
            serde_json::Value::String(group_id.to_string()),
        );
        client_invoke(self.inner, "VaultsRevokeGroupPermissions", p)?;
        Ok(())
    }
}
