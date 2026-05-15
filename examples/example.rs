use onepassword_sdk::{
    AutofillBehavior, Client, DocumentCreateParams, FileCreateParams, Item, ItemCategory,
    ItemCreateParams, ItemField, ItemFieldType, ItemListFilter, ItemListFilterByStateInner,
    ItemSection, ItemShareDuration, ItemShareParams, ItemsApi, ItemsFilesApi, ItemsSharesApi,
    PasswordRecipe, PasswordRecipeMemorableInner, PasswordRecipePinInner,
    PasswordRecipeRandomInner, SdkError, Secrets, SecretsApi, SeparatorType, VaultCreateParams,
    VaultGetParams, VaultUpdateParams, VaultsApi, Website, WordListType,
};

fn main() -> Result<(), SdkError> {
    let token = std::env::var("OP_SERVICE_ACCOUNT_TOKEN")
        .expect("Set OP_SERVICE_ACCOUNT_TOKEN environment variable");
    let vault_id =
        std::env::var("OP_VAULT_ID").expect("Set OP_VAULT_ID to a vault ID in your account");

    let client = Client::builder()
        .service_account_token(&token)
        .integration_info("1Password Rust SDK Example", "v1.0.0")
        .build()?;

    // --- Passwords (no client state needed) ---
    validate_secret_references();
    generate_passwords()?;

    // --- Items (create first, then resolve secrets from it) ---
    let item = create_item(&client, &vault_id)?;
    let item_id = item.id.clone();
    let item_vault = item.vault_id.clone();

    // Run item operations, capturing any error
    let item_result = (|| -> Result<(), SdkError> {
        resolve_secrets(&client, &item)?;
        resolve_all_secrets(&client, &item)?;
        list_items(&client, &vault_id)?;
        let item = update_item(&client, item)?;
        share_item(&client, &vault_id, &item)?;
        create_ssh_key_item(&client, &vault_id)?;
        create_and_replace_document_item(&client, &vault_id)?;
        create_attach_and_delete_file_field_item(&client, &vault_id)?;
        batch_item_operations(&client, &vault_id)?;
        // Archive and delete use separate items to demonstrate the distinction
        let archive_target = client.items().create(ItemCreateParams {
            category: ItemCategory::SecureNote,
            vault_id: vault_id.to_string(),
            title: "Archive Demo".to_string(),
            fields: None,
            sections: None,
            notes: Some("This item will be archived".to_string()),
            tags: None,
            websites: None,
            files: None,
            document: None,
        })?;
        archive_item(&client, &vault_id, &archive_target.id)?;
        // Archived items can still be deleted
        client.items().delete(&vault_id, &archive_target.id)?;
        println!("Archived and deleted demo item");

        // Delete the main example item
        client.items().delete(&vault_id, &item.id)?;
        println!("Deleted main item: {}", item.id);
        Ok(())
    })();

    // Best-effort cleanup if operations failed partway
    if item_result.is_err() {
        let _ = client.items().delete(&item_vault, &item_id);
    }

    // --- Vaults ---
    vault_operations(&client)?;

    match item_result {
        Ok(()) => println!("\nAll examples completed successfully."),
        Err(e) => {
            println!("\nItem operations failed (cleanup attempted): {e}");
            return Err(e);
        }
    }

    Ok(())
}

// ── Secrets ──────────────────────────────────────────────────────────────

fn resolve_secrets(client: &Client, item: &Item) -> Result<(), SdkError> {
    println!("\n=== Resolve Secrets ===");

    // Resolve a secret using a reference built from the created item
    let secret_ref = format!("op://{}/{}/username", item.vault_id, item.id);
    client.secrets().resolve(&secret_ref)?;
    println!("Secret resolved successfully");

    Ok(())
}

fn resolve_all_secrets(client: &Client, item: &Item) -> Result<(), SdkError> {
    println!("\n=== Resolve All Secrets ===");

    let references = vec![
        format!("op://{}/{}/username", item.vault_id, item.id),
        format!("op://{}/{}/password", item.vault_id, item.id),
    ];

    let response = client.secrets().resolve_all(&references)?;

    for (reference, result) in &response.individual_responses {
        if result.content.is_some() {
            println!("{reference}: resolved successfully");
        }
        if let Some(error) = &result.error {
            println!("{reference}: error: {error:?}");
        }
    }

    Ok(())
}

fn validate_secret_references() {
    println!("\n=== Validate Secret References ===");

    match Secrets::validate_secret_reference("op://vault/item/field") {
        Ok(()) => println!("Valid reference"),
        Err(e) => println!("Invalid reference: {e}"),
    }

    match Secrets::validate_secret_reference("not-a-valid-reference") {
        Ok(()) => println!("Valid reference"),
        Err(e) => println!("Invalid reference (expected): {e}"),
    }
}

fn generate_passwords() -> Result<(), SdkError> {
    println!("\n=== Generate Passwords ===");

    // PIN
    Secrets::generate_password(PasswordRecipe::Pin(PasswordRecipePinInner { length: 8 }))?;
    println!("PIN generated successfully");

    // Memorable password
    Secrets::generate_password(PasswordRecipe::Memorable(PasswordRecipeMemorableInner {
        separator_type: SeparatorType::Hyphens,
        capitalize: true,
        word_list_type: WordListType::FullWords,
        word_count: 4,
    }))?;
    println!("Memorable password generated successfully");

    // Random password
    Secrets::generate_password(PasswordRecipe::Random(PasswordRecipeRandomInner {
        include_digits: true,
        include_symbols: true,
        length: 32,
    }))?;
    println!("Random password generated successfully");

    Ok(())
}

// ── Items ────────────────────────────────────────────────────────────────

fn create_item(client: &Client, vault_id: &str) -> Result<Item, SdkError> {
    println!("\n=== Create Item ===");

    // Generate a password for the example item using the SDK
    let password = Secrets::generate_password(PasswordRecipe::Random(PasswordRecipeRandomInner {
        include_digits: true,
        include_symbols: true,
        length: 20,
    }))?;

    let item = client.items().create(ItemCreateParams {
        category: ItemCategory::Login,
        vault_id: vault_id.to_string(),
        title: "Rust SDK Example Login".to_string(),
        fields: Some(vec![
            ItemField {
                id: "username".to_string(),
                title: "username".to_string(),
                section_id: None,
                field_type: ItemFieldType::Text,
                value: "user@example.com".to_string(),
                details: None,
            },
            ItemField {
                id: "password".to_string(),
                title: "password".to_string(),
                section_id: None,
                field_type: ItemFieldType::Concealed,
                value: password.password,
                details: None,
            },
            ItemField {
                id: "api_key".to_string(),
                title: "API Key".to_string(),
                section_id: Some("additional".to_string()),
                field_type: ItemFieldType::Concealed,
                value: "example-api-key-placeholder".to_string(),
                details: None,
            },
        ]),
        sections: Some(vec![ItemSection {
            id: "additional".to_string(),
            title: "Additional Details".to_string(),
        }]),
        notes: Some("Created by Rust SDK example".to_string()),
        tags: Some(vec!["example".to_string(), "rust-sdk".to_string()]),
        websites: Some(vec![Website {
            url: "https://example.com".to_string(),
            label: "Example".to_string(),
            autofill_behavior: AutofillBehavior::AnywhereOnWebsite,
        }]),
        files: None,
        document: None,
    })?;

    println!("Created item: {} ({})", item.title, item.id);
    Ok(item)
}

fn list_items(client: &Client, vault_id: &str) -> Result<(), SdkError> {
    println!("\n=== List Items ===");

    // List all active items
    let items = client.items().list(vault_id, &[])?;
    println!("Total items in vault: {}", items.len());

    // List only active items (exclude archived)
    let active_items = client.items().list(
        vault_id,
        &[ItemListFilter::ByState(ItemListFilterByStateInner {
            active: true,
            archived: false,
        })],
    )?;
    println!("Active items: {}", active_items.len());

    Ok(())
}

fn update_item(client: &Client, mut item: Item) -> Result<Item, SdkError> {
    println!("\n=== Update Item ===");

    item.title = "Rust SDK Example Login (Updated)".to_string();
    item.tags.push("updated".to_string());

    if let Some(field) = item.fields.iter_mut().find(|f| f.id == "username") {
        field.value = "updated-user@example.com".to_string();
    }

    let updated = client.items().put(item)?;
    println!("Updated item: {} (v{})", updated.title, updated.version);
    Ok(updated)
}

fn archive_item(client: &Client, vault_id: &str, item_id: &str) -> Result<(), SdkError> {
    println!("\n=== Archive Item ===");
    client.items().archive(vault_id, item_id)?;
    println!("Archived item: {item_id}");
    Ok(())
}

fn share_item(client: &Client, vault_id: &str, item: &Item) -> Result<(), SdkError> {
    println!("\n=== Share Item ===");

    let recipient = match std::env::var("OP_SHARE_RECIPIENT") {
        Ok(r) => r,
        Err(_) => {
            println!("Skipping (set OP_SHARE_RECIPIENT to an email address to test)");
            return Ok(());
        }
    };

    let policy = client
        .items()
        .shares()
        .get_account_policy(vault_id, &item.id)?;
    println!("Account policy max expiry: {:?}", policy.max_expiry);

    let valid_recipients = client
        .items()
        .shares()
        .validate_recipients(policy.clone(), &[recipient])?;
    println!("Valid recipients: {}", valid_recipients.len());

    let _share_link = client.items().shares().create(
        item.clone(),
        policy,
        ItemShareParams {
            recipients: Some(valid_recipients),
            expire_after: Some(ItemShareDuration::OneDay),
            one_time_only: true,
        },
    )?;
    // Don't log the share link — it's a bearer URL
    println!("Share link created successfully");

    Ok(())
}

fn batch_item_operations(client: &Client, vault_id: &str) -> Result<(), SdkError> {
    println!("\n=== Batch Item Operations ===");

    // Batch create
    let params: Vec<ItemCreateParams> = (1..=3)
        .map(|i| ItemCreateParams {
            category: ItemCategory::SecureNote,
            vault_id: vault_id.to_string(),
            title: format!("Batch Note {i}"),
            fields: None,
            sections: None,
            notes: Some(format!("Batch-created note #{i}")),
            tags: Some(vec!["batch".to_string()]),
            websites: None,
            files: None,
            document: None,
        })
        .collect();

    let create_response = client.items().create_all(vault_id, &params)?;
    let mut created_ids = Vec::new();
    for resp in &create_response.individual_responses {
        if let Some(item) = &resp.content {
            println!("  Created: {} ({})", item.title, item.id);
            created_ids.push(item.id.clone());
        }
        if let Some(err) = &resp.error {
            println!("  Error: {err:?}");
        }
    }

    // Batch get
    let get_response = client.items().get_all(vault_id, &created_ids)?;
    println!(
        "Batch get returned {} results",
        get_response.individual_responses.len()
    );

    // Batch delete
    let delete_response = client.items().delete_all(vault_id, &created_ids)?;
    for (id, resp) in &delete_response.individual_responses {
        if resp.error.is_some() {
            println!("  Failed to delete {id}: {:?}", resp.error);
        }
    }
    println!("Batch deleted {} items", created_ids.len());

    Ok(())
}

fn create_ssh_key_item(client: &Client, vault_id: &str) -> Result<(), SdkError> {
    println!("\n=== Create SSH Key Item ===");

    let private_key = generate_ed25519_key()?;

    let item = client.items().create(ItemCreateParams {
        category: ItemCategory::SshKey,
        vault_id: vault_id.to_string(),
        title: "Rust SDK Example SSH Key".to_string(),
        fields: Some(vec![ItemField {
            id: "private_key".to_string(),
            title: "private key".to_string(),
            section_id: None,
            field_type: ItemFieldType::SshKey,
            value: private_key,
            details: None,
        }]),
        sections: None,
        notes: None,
        tags: Some(vec!["ssh".to_string(), "example".to_string()]),
        websites: None,
        files: None,
        document: None,
    })?;

    println!("Created SSH key item: {} ({})", item.title, item.id);

    // Clean up
    client.items().delete(vault_id, &item.id)?;
    Ok(())
}

fn generate_ed25519_key() -> Result<String, SdkError> {
    let dir = std::env::temp_dir().join(format!("op-sdk-example-{}", std::process::id()));
    let key_path = dir.join("key");

    std::fs::create_dir_all(&dir)
        .map_err(|e| SdkError::Config(format!("failed to create temp dir: {e}")))?;

    let result = (|| {
        let status = std::process::Command::new("ssh-keygen")
            .args(["-t", "ed25519", "-f"])
            .arg(&key_path)
            .args(["-N", "", "-q"])
            .status()
            .map_err(|e| SdkError::Config(format!("failed to run ssh-keygen: {e}")))?;

        if !status.success() {
            return Err(SdkError::Config("ssh-keygen failed".to_string()));
        }

        std::fs::read_to_string(&key_path)
            .map_err(|e| SdkError::Config(format!("failed to read generated key: {e}")))
    })();

    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn create_and_replace_document_item(client: &Client, vault_id: &str) -> Result<(), SdkError> {
    println!("\n=== Document Item ===");

    // Create a document item
    let item = client.items().create(ItemCreateParams {
        category: ItemCategory::Document,
        vault_id: vault_id.to_string(),
        title: "Rust SDK Example Document".to_string(),
        fields: None,
        sections: None,
        notes: None,
        tags: None,
        websites: None,
        files: None,
        document: Some(DocumentCreateParams {
            name: "config.json".to_string(),
            content: br#"{"key": "value"}"#.to_vec(),
        }),
    })?;
    println!("Created document item: {} ({})", item.title, item.id);

    // Replace the document content
    let updated = client.items().files().replace_document(
        item,
        DocumentCreateParams {
            name: "config.json".to_string(),
            content: br#"{"key": "updated-value", "version": 2}"#.to_vec(),
        },
    )?;
    println!("Replaced document in item: {}", updated.id);

    // Read back the document
    if let Some(doc) = &updated.document {
        let content = client
            .items()
            .files()
            .read(&vault_id, &updated.id, doc.clone())?;
        println!("Document content length: {} bytes", content.len());
    }

    // Clean up
    client.items().delete(vault_id, &updated.id)?;
    Ok(())
}

fn create_attach_and_delete_file_field_item(
    client: &Client,
    vault_id: &str,
) -> Result<(), SdkError> {
    println!("\n=== File Field Item ===");

    let section_id = "attachments";
    let field_id = "config_file";

    // Create an item with a section for file attachments
    let item = client.items().create(ItemCreateParams {
        category: ItemCategory::SecureNote,
        vault_id: vault_id.to_string(),
        title: "Rust SDK Example File Attachment".to_string(),
        fields: None,
        sections: Some(vec![ItemSection {
            id: section_id.to_string(),
            title: "Attachments".to_string(),
        }]),
        notes: None,
        tags: None,
        websites: None,
        files: None,
        document: None,
    })?;
    println!("Created item: {} ({})", item.title, item.id);

    // Attach a file
    let item = client.items().files().attach(
        item,
        FileCreateParams {
            name: "notes.txt".to_string(),
            content: b"These are some important notes.".to_vec(),
            section_id: section_id.to_string(),
            field_id: field_id.to_string(),
        },
    )?;
    println!("Attached file to item");

    // Read the file back
    let file = item.files.first().expect("Item should have a file");
    let content = client
        .items()
        .files()
        .read(vault_id, &item.id, file.attributes.clone())?;
    let file_section_id = file.section_id.clone();
    let file_field_id = file.field_id.clone();
    println!(
        "Read file '{}': {} bytes",
        file.attributes.name,
        content.len()
    );

    // Delete the file field
    let item = client
        .items()
        .files()
        .delete(item, &file_section_id, &file_field_id)?;
    println!("Deleted file field from item");

    // Clean up
    client.items().delete(vault_id, &item.id)?;
    Ok(())
}

// ── Vaults ───────────────────────────────────────────────────────────────

fn vault_operations(client: &Client) -> Result<(), SdkError> {
    println!("\n=== Vault Operations ===");

    // List vaults
    let vaults = client.vaults().list(None)?;
    println!("Total vaults: {}", vaults.len());
    for vault in &vaults {
        println!(
            "  - {} ({:?}, {} items)",
            vault.title, vault.vault_type, vault.active_item_count
        );
    }

    // Create a vault
    let vault = client.vaults().create(VaultCreateParams {
        title: "Rust SDK Example Vault".to_string(),
        description: Some("Created by the Rust SDK example".to_string()),
        allow_admins_access: Some(true),
    })?;
    println!("Created vault: {} ({})", vault.title, vault.id);

    // Get vault overview
    let overview = client.vaults().get_overview(&vault.id)?;
    println!(
        "Vault overview: {} — {} items",
        overview.title, overview.active_item_count
    );

    // Get full vault details with accessors
    let full_vault = client.vaults().get(
        &vault.id,
        VaultGetParams {
            accessors: Some(true),
        },
    )?;
    println!(
        "Vault accessors: {}",
        full_vault.access.as_ref().map_or(0, |a| a.len())
    );

    // Update vault
    let updated = client.vaults().update(
        &vault.id,
        VaultUpdateParams {
            title: Some("Rust SDK Example Vault (Updated)".to_string()),
            description: Some("Updated description".to_string()),
        },
    )?;
    println!("Updated vault: {}", updated.title);

    // Delete vault
    client.vaults().delete(&vault.id)?;
    println!("Deleted vault: {}", vault.id);

    Ok(())
}
