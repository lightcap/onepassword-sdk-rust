#[cfg(feature = "desktop")]
fn main() {
    use onepassword_sdk::{
        Client, GroupAccess, GroupGetParams, GroupVaultAccess, GroupsApi, ItemCategory,
        ItemCreateParams, ItemsApi, SdkError, SecretsApi, VaultCreateParams, VaultGetParams,
        VaultUpdateParams, VaultsApi, permissions,
    };

    let account = std::env::var("OP_ACCOUNT")
        .expect("Set OP_ACCOUNT to your 1Password sign-in address (e.g. my.1password.com)");
    let vault_id =
        std::env::var("OP_VAULT_ID").expect("Set OP_VAULT_ID to a vault ID in your account");
    let group_id = std::env::var("OP_GROUP_ID").ok();

    let client = Client::builder()
        .desktop_app_integration(&account)
        .integration_info("1Password Rust SDK Example", "v1.0.0")
        .build()
        .expect("Failed to create client — is 1Password desktop app running?");

    // --- Resolve a secret ---
    let secret_ref = std::env::var("OP_SECRET_REF")
        .unwrap_or_else(|_| "op://MyVault/Example/username".to_string());
    client
        .secrets()
        .resolve(&secret_ref)
        .expect("Failed to resolve secret");
    println!("Secret resolved successfully");

    // --- List vaults ---
    let vaults = client.vaults().list(None).expect("Failed to list vaults");
    println!("\nVaults ({}):", vaults.len());
    for vault in &vaults {
        println!("  - {} ({:?})", vault.title, vault.vault_type);
    }

    // --- List items ---
    let items = client
        .items()
        .list(&vault_id, &[])
        .expect("Failed to list items");
    println!("\nItems in vault ({}):", items.len());
    for item in items.iter().take(5) {
        println!("  - {} ({:?})", item.title, item.category);
    }

    // --- Vault CRUD ---
    vault_operations(&client);

    // --- Batch item operations ---
    batch_item_operations(&client, &vault_id);

    // --- Group permissions ---
    group_permission_operations(&client, &group_id).expect("Group permission operations failed");

    println!("\nAll desktop app examples completed successfully.");

    fn vault_operations(client: &Client) {
        println!("\n=== Vault Operations ===");

        let vault = client
            .vaults()
            .create(VaultCreateParams {
                title: "Desktop SDK Example Vault".to_string(),
                description: Some("Created via desktop app integration".to_string()),
                allow_admins_access: Some(true),
            })
            .expect("Failed to create vault");
        println!("Created vault: {} ({})", vault.title, vault.id);

        let overview = client
            .vaults()
            .get_overview(&vault.id)
            .expect("Failed to get vault overview");
        println!(
            "Vault: {} — {} items",
            overview.title, overview.active_item_count
        );

        let full = client
            .vaults()
            .get(
                &vault.id,
                VaultGetParams {
                    accessors: Some(true),
                },
            )
            .expect("Failed to get vault details");
        println!(
            "Vault accessors: {}",
            full.access.as_ref().map_or(0, |a| a.len())
        );

        let updated = client
            .vaults()
            .update(
                &vault.id,
                VaultUpdateParams {
                    title: Some("Desktop SDK Example Vault (Updated)".to_string()),
                    description: None,
                },
            )
            .expect("Failed to update vault");
        println!("Updated vault: {}", updated.title);

        client
            .vaults()
            .delete(&vault.id)
            .expect("Failed to delete vault");
        println!("Deleted vault: {}", vault.id);
    }

    fn batch_item_operations(client: &Client, vault_id: &str) {
        println!("\n=== Batch Item Operations ===");

        let params: Vec<ItemCreateParams> = (1..=3)
            .map(|i| ItemCreateParams {
                category: ItemCategory::SecureNote,
                vault_id: vault_id.to_string(),
                title: format!("Desktop Batch Note {i}"),
                fields: None,
                sections: None,
                notes: Some(format!("Batch note #{i} via desktop app")),
                tags: Some(vec!["desktop-batch".to_string()]),
                websites: None,
                files: None,
                document: None,
            })
            .collect();

        let response = client
            .items()
            .create_all(vault_id, &params)
            .expect("Failed to batch create items");

        let mut created_ids = Vec::new();
        for resp in &response.individual_responses {
            match (&resp.content, &resp.error) {
                (Some(item), _) => {
                    println!("  Created: {} ({})", item.title, item.id);
                    created_ids.push(item.id.clone());
                }
                (_, Some(err)) => println!("  Error: {err:?}"),
                _ => {}
            }
        }

        let get_response = client
            .items()
            .get_all(vault_id, &created_ids)
            .expect("Failed to batch get items");
        for resp in &get_response.individual_responses {
            match (&resp.content, &resp.error) {
                (Some(item), _) => println!("  Got: {} (v{})", item.title, item.version),
                (_, Some(err)) => println!("  Error: {err:?}"),
                _ => {}
            }
        }

        let delete_response = client
            .items()
            .delete_all(vault_id, &created_ids)
            .expect("Failed to batch delete items");
        for (id, resp) in &delete_response.individual_responses {
            if let Some(err) = &resp.error {
                println!("  Failed to delete {id}: {err:?}");
            }
        }
        println!("Batch deleted {} items", created_ids.len());
    }

    fn group_permission_operations(
        client: &Client,
        group_id: &Option<String>,
    ) -> Result<(), SdkError> {
        println!("\n=== Group Permission Operations ===");

        let group_id = match group_id {
            Some(id) => id,
            None => {
                println!("Skipping group permissions (set OP_GROUP_ID to test)");
                return Ok(());
            }
        };

        // Get group info
        let group = client.groups().get(
            group_id,
            GroupGetParams {
                vault_permissions: Some(true),
            },
        )?;
        println!("Group: {} ({:?})", group.title, group.group_type);

        // Create a temporary vault for the permissions demo
        let temp_vault = client.vaults().create(VaultCreateParams {
            title: "SDK Permissions Demo (temp)".to_string(),
            description: Some("Temporary vault for group permissions demo".to_string()),
            allow_admins_access: Some(true),
        })?;
        let temp_vault_id = temp_vault.id.clone();
        println!(
            "Created temp vault: {} ({})",
            temp_vault.title, temp_vault_id
        );

        let result = (|| -> Result<(), SdkError> {
            // Grant read + reveal permissions to the group on the temp vault
            client.vaults().grant_group_permissions(
                &temp_vault_id,
                &[GroupAccess {
                    group_id: group_id.to_string(),
                    permissions: permissions::READ_ITEMS | permissions::REVEAL_ITEM_PASSWORD,
                }],
            )?;
            println!("Granted read permissions to group on vault");

            // Update to also allow creating items
            client
                .vaults()
                .update_group_permissions(&[GroupVaultAccess {
                    vault_id: temp_vault_id.clone(),
                    group_id: group_id.to_string(),
                    permissions: permissions::READ_ITEMS
                        | permissions::REVEAL_ITEM_PASSWORD
                        | permissions::CREATE_ITEMS,
                }])?;
            println!("Updated group permissions to include create");

            // Revoke all permissions
            client
                .vaults()
                .revoke_group_permissions(&temp_vault_id, group_id)?;
            println!("Revoked group permissions");

            Ok(())
        })();

        // Always clean up the temp vault (best-effort)
        let _ = client.vaults().delete(&temp_vault_id);
        println!("Deleted temp vault: {temp_vault_id}");

        result
    }
}

#[cfg(not(feature = "desktop"))]
fn main() {
    eprintln!(
        "This example requires the 'desktop' feature: cargo run --example desktop_app --features desktop"
    );
}
