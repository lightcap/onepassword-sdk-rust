<p align="center">
  <a href="https://1password.com">
      <h1 align="center">1Password Rust SDK</h1>
  </a>
</p>

<p align="center">
 <h4 align="center">Build integrations that programmatically interact with 1Password.</h4>
</p>

<p align="center">
  <a href="https://developer.1password.com/docs/sdks/">Documentation</a> | <a href="https://github.com/1Password/onepassword-sdk-rust/tree/main/examples">Examples</a>
</p>

---

> **Note:** This is an unofficial, community-maintained Rust SDK for the 1Password platform.

## Requirements

- Rust 1.93.1 or later (edition 2024)
- A [1Password account](https://1password.com/) with either:
  - A [service account](https://developer.1password.com/docs/service-accounts/) token, or
  - The [1Password desktop app](https://1password.com/downloads/) installed

## Get started

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
onepassword-sdk = { git = "https://github.com/lightcap/onepassword-sdk-rust" }
```

You can choose between two [authentication methods](https://developer.1password.com/docs/sdks/concepts#authentication): local authorization prompts from the [1Password desktop app](#option-1-1password-desktop-app) or automated authentication with a [1Password Service Account](#option-2-1password-service-account).

### Option 1: 1Password desktop app

[1Password desktop app authentication](https://developer.1password.com/docs/sdks/concepts#1password-desktop-app) is best for local integrations that require minimal setup from end users and sensitive workflows that require human-in-the-loop approval.

1. Install the [1Password desktop app](https://1password.com/downloads/) and sign in to your account.
2. Navigate to **Settings** > **Developer**.
3. Under Integrate with the 1Password SDKs, select **Integrate with other apps**.
4. If you want to authenticate with biometrics, navigate to **Settings** > **Security**, then turn on the option to unlock using [Touch ID](https://support.1password.com/touch-id-mac/), [Windows Hello](https://support.1password.com/windows-hello/), or [system authentication](https://support.1password.com/system-authentication-linux/).
5. Enable the `desktop` feature in your `Cargo.toml`:

   ```toml
   [dependencies]
   onepassword-sdk = { git = "https://github.com/lightcap/onepassword-sdk-rust", default-features = false, features = ["desktop"] }
   ```

6. Use the SDK in your project, replacing `your-account-name` with the name of your 1Password account:

```rust
use onepassword_sdk::{Client, SecretsApi};

fn main() {
    let client = Client::builder()
        .desktop_app_integration("your-account-name")
        .integration_info("My 1Password Integration", "v1.0.0")
        .build()
        .expect("Failed to create client");

    let secret = client
        .secrets()
        .resolve("op://vault/item/field")
        .expect("Failed to resolve secret");

    println!("Secret resolved successfully");
}
```

### Option 2: 1Password Service Account

[Service account authentication](https://developer.1password.com/docs/sdks/concepts/#1password-service-account) is best for automated access and limiting your integration to least privilege access.

1. [Create a service account](https://my.1password.com/developer-tools/infrastructure-secrets/serviceaccount/) and give it the appropriate permissions in the vaults where the items you want to use with the SDK are saved.
2. Provision your service account token. We recommend provisioning your token from the environment. For example:

   **macOS or Linux**

   ```bash
   export OP_SERVICE_ACCOUNT_TOKEN=<your-service-account-token>
   ```

   **Windows**

   ```powershell
   $Env:OP_SERVICE_ACCOUNT_TOKEN = "<your-service-account-token>"
   ```

3. Use the SDK in your project:

```rust
use onepassword_sdk::{Client, SecretsApi};

fn main() {
    let token = std::env::var("OP_SERVICE_ACCOUNT_TOKEN")
        .expect("Set OP_SERVICE_ACCOUNT_TOKEN environment variable");

    let client = Client::builder()
        .service_account_token(&token)
        .integration_info("My 1Password Integration", "v1.0.0")
        .build()
        .expect("Failed to create client");

    let secret = client
        .secrets()
        .resolve("op://vault/item/field")
        .expect("Failed to resolve secret");

    println!("Secret resolved successfully");
}
```

Make sure to use [secret reference URIs](https://developer.1password.com/docs/cli/secret-reference-syntax/) with the syntax `op://vault/item/field` to securely load secrets from 1Password into your code.

## Usage

### Resolve secrets

Use secret references to load individual secrets without retrieving full items:

```rust
use onepassword_sdk::{Client, SecretsApi};

let secret = client.secrets().resolve("op://vault/item/field")?;
```

Resolve multiple secrets at once:

```rust
let refs = vec![
    "op://vault/item/username".to_string(),
    "op://vault/item/password".to_string(),
];
let resolved = client.secrets().resolve_all(&refs)?;
```

Validate a secret reference URI without resolving it (no client required):

```rust
use onepassword_sdk::Secrets;

Secrets::validate_secret_reference("op://vault/item/field")?;
```

### Manage items

Create, retrieve, update, delete, archive, and list items:

```rust
use onepassword_sdk::{Client, ItemsApi, ItemCreateParams, ItemCategory, ItemField, ItemFieldType};

// Create an item
let params = ItemCreateParams {
    title: "My Login".to_string(),
    category: ItemCategory::Login,
    vault_id: "vault-uuid".to_string(),
    fields: Some(vec![
        ItemField {
            id: "username".to_string(),
            title: "Username".to_string(),
            value: "user@example.com".to_string(),
            field_type: ItemFieldType::Text,
            section_id: None,
            details: None,
        },
        ItemField {
            id: "password".to_string(),
            title: "Password".to_string(),
            value: "my-secret-password".to_string(),
            field_type: ItemFieldType::Concealed,
            section_id: None,
            details: None,
        },
    ]),
    sections: None,
    notes: None,
    tags: None,
    websites: None,
    files: None,
    document: None,
};
let item = client.items().create(params)?;

// Get an item
let item = client.items().get("vault-uuid", "item-uuid")?;

// Update an item
let updated = client.items().put(item)?;

// List items
let overviews = client.items().list("vault-uuid", &[])?;

// Archive an item
client.items().archive("vault-uuid", "item-uuid")?;

// Delete an item
client.items().delete("vault-uuid", "item-uuid")?;
```

#### Batch operations

Create, get, or delete multiple items in a single call:

```rust
// Create multiple items at once
let response = client.items().create_all("vault-uuid", &[params1, params2])?;

// Get multiple items by ID
let item_ids = vec!["item-1".to_string(), "item-2".to_string()];
let response = client.items().get_all("vault-uuid", &item_ids)?;

// Delete multiple items
let response = client.items().delete_all("vault-uuid", &item_ids)?;
```

### Manage files

Attach, read, delete files and replace documents on items:

```rust
use onepassword_sdk::{ItemsApi, ItemsFilesApi, FileCreateParams, FileAttributes, DocumentCreateParams};

// Attach a file to an item
let file_params = FileCreateParams {
    name: "config.json".to_string(),
    content: std::fs::read("config.json")?,
    section_id: "section-uuid".to_string(),
    field_id: "field-uuid".to_string(),
};
let updated_item = client.items().files().attach(item, file_params)?;

// Read a file
let content: Vec<u8> = client.items().files().read(
    "vault-uuid",
    "item-uuid",
    FileAttributes { name: "config.json".to_string(), id: "file-uuid".to_string(), size: 1024 },
)?;

// Delete a file from an item
let updated_item = client.items().files().delete(item, "section-uuid", "field-uuid")?;

// Replace a document item's content
let doc_params = DocumentCreateParams {
    name: "updated-doc.pdf".to_string(),
    content: std::fs::read("updated-doc.pdf")?,
};
let updated_item = client.items().files().replace_document(item, doc_params)?;
```

### Share items

Share items securely with other users:

```rust
use onepassword_sdk::{ItemsApi, ItemsSharesApi, ItemShareParams, ItemShareDuration};

// Get the sharing policy for the item
let policy = client.items().shares().get_account_policy("vault-uuid", "item-uuid")?;

// Validate recipients
let recipients = client.items().shares().validate_recipients(
    policy.clone(),
    &["recipient@example.com".to_string()],
)?;

// Create the share
let share_params = ItemShareParams {
    recipients: Some(recipients),
    expire_after: Some(ItemShareDuration::SevenDays),
    one_time_only: false,
};
let share_link = client.items().shares().create(item, policy, share_params)?;
```

### Generate passwords

Generate passwords without requiring a client:

```rust
use onepassword_sdk::{Secrets, PasswordRecipe, PasswordRecipeRandomInner};

// Generate a random password
let response = Secrets::generate_password(
    PasswordRecipe::Random(PasswordRecipeRandomInner {
        length: 32,
        include_digits: true,
        include_symbols: true,
    }),
)?;
println!("Password generated successfully");
```

### Manage vaults

Create, retrieve, update, delete, and list vaults:

```rust
use onepassword_sdk::{Client, VaultsApi, VaultCreateParams, VaultGetParams, VaultUpdateParams};

// Create a vault
let params = VaultCreateParams {
    title: "Engineering Secrets".to_string(),
    description: Some("Shared secrets for the engineering team".to_string()),
    allow_admins_access: Some(true),
};
let vault = client.vaults().create(params)?;

// Get a vault overview (lightweight)
let overview = client.vaults().get_overview("vault-uuid")?;

// Get full vault details (with optional accessor info)
let vault = client.vaults().get("vault-uuid", VaultGetParams { accessors: Some(true) })?;

// Update a vault
let updated = client.vaults().update("vault-uuid", VaultUpdateParams {
    title: Some("Renamed Vault".to_string()),
    description: None,
})?;

// List vaults
let vault_overviews = client.vaults().list(None)?;

// Delete a vault
client.vaults().delete("vault-uuid")?;
```

### Manage vault permissions

Grant, update, and revoke group permissions on vaults:

```rust
use onepassword_sdk::{VaultsApi, GroupAccess, GroupVaultAccess, permissions};

// Grant group access to a vault
let access = GroupAccess {
    group_id: "group-uuid".to_string(),
    permissions: permissions::READ_ITEMS | permissions::CREATE_ITEMS,
};
client.vaults().grant_group_permissions("vault-uuid", &[access])?;

// Update existing group permissions
let updated_access = GroupVaultAccess {
    vault_id: "vault-uuid".to_string(),
    group_id: "group-uuid".to_string(),
    permissions: permissions::READ_ITEMS | permissions::UPDATE_ITEMS | permissions::CREATE_ITEMS,
};
client.vaults().update_group_permissions(&[updated_access])?;

// Revoke group access
client.vaults().revoke_group_permissions("vault-uuid", "group-uuid")?;
```

### Retrieve groups

```rust
use onepassword_sdk::{GroupsApi, GroupGetParams};

let group = client.groups().get("group-uuid", GroupGetParams {
    vault_permissions: Some(true),
})?;
```

### Read 1Password Environments (beta)

```rust
use onepassword_sdk::EnvironmentsApi;

let response = client.environments().get_variables("environment-uuid")?;
for var in response.variables {
    println!("{}: {}", var.name, var.value);
}
```

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `wasm` | Yes | Embedded WASM core via [Extism](https://extism.org/). Use for standalone applications with service account authentication. |
| `desktop` | No | Native shared library integration with the 1Password desktop app. Use for local development and user-facing applications. |

## Error handling

All SDK operations return `Result<T, SdkError>`. The error variants are:

| Variant | Description |
|---------|-------------|
| `Core` | Error from the 1Password core engine |
| `Serialization` | JSON serialization/deserialization failure |
| `Plugin` | WASM plugin runtime error |
| `Config` | Client configuration error (e.g., missing token) |
| `DesktopSessionExpired` | Desktop app session expired (automatically retried) |
| `RateLimitExceeded` | API rate limit hit |
| `SharedLib` | Shared library loading or call error |

```rust
use onepassword_sdk::{Client, SecretsApi, SdkError};

match client.secrets().resolve("op://vault/item/field") {
    Ok(_) => println!("Secret resolved successfully"),
    Err(SdkError::Core { name, message }) => {
        eprintln!("1Password error ({}): {}", name, message);
    }
    Err(e) => eprintln!("Error: {e}"),
}
```

## Supported functionality

### Item management

**Operations:**

- [x] [Retrieve secrets](https://developer.1password.com/docs/sdks/load-secrets)
- [x] [Retrieve items](https://developer.1password.com/docs/sdks/manage-items#get-an-item)
- [x] [Create items](https://developer.1password.com/docs/sdks/manage-items#create-an-item)
- [x] [Update items](https://developer.1password.com/docs/sdks/manage-items#update-an-item)
- [x] [Delete items](https://developer.1password.com/docs/sdks/manage-items#delete-an-item)
- [x] [Archive items](https://developer.1password.com/docs/sdks/manage-items/#archive-an-item)
- [x] [List items](https://developer.1password.com/docs/sdks/list-vaults-items/)
- [x] [Share items](https://developer.1password.com/docs/sdks/share-items)
- [x] [Generate PIN, random, and memorable passwords](https://developer.1password.com/docs/sdks/manage-items#generate-a-password)

**Field types:**

- [x] API Keys, Passwords, Concealed fields, Text fields, Notes
- [x] SSH private keys, public keys, fingerprint and key type
- [x] One-time passwords
- [x] URLs, Websites (used to suggest and autofill logins)
- [x] Phone numbers, Credit card types/numbers, Emails
- [x] References to other items
- [x] Address, Date, MM/YY
- [x] File attachments and Document items
- [x] Menu
- [ ] Passkeys

### Vault management

- [x] [Retrieve vaults](https://developer.1password.com/docs/sdks/vaults#get-a-vault-overview)
- [x] [Create vaults](https://developer.1password.com/docs/sdks/vaults#create-a-vault)
- [x] [Update vaults](https://developer.1password.com/docs/sdks/vaults#update-a-vault)
- [x] [Delete vaults](https://developer.1password.com/docs/sdks/vaults#delete-a-vault)
- [x] [List vaults](https://developer.1password.com/docs/sdks/list-vaults-items#list-vaults)
- [x] [Manage group vault permissions](https://developer.1password.com/docs/sdks/vault-permissions)
- [ ] Manage user vault permissions

### User & access management

- [x] [Retrieve groups](https://developer.1password.com/docs/sdks/groups/)
- [ ] Provision, Retrieve, List, Suspend users
- [ ] List, Create groups, Update group membership

### Environments management

- [x] [Read 1Password Environments](https://developer.1password.com/docs/sdks/environments) (beta)

### Compliance & reporting

- [ ] Watchtower insights
- [ ] Travel mode
- [ ] Events (use [1Password Events Reporting API](https://developer.1password.com/docs/events-api/) directly)

### Authentication

- [x] [1Password Service Accounts](https://developer.1password.com/docs/sdks/concepts#1password-service-account)
- [x] [User authentication (desktop app)](https://developer.1password.com/docs/sdks/concepts#1password-desktop-app)
- [ ] 1Password Connect

## Learn more

- [Load secrets](https://developer.1password.com/docs/sdks/load-secrets)
- [Read 1Password Environments (beta)](https://developer.1password.com/docs/sdks/environments)
- [Manage items](https://developer.1password.com/docs/sdks/manage-items)
- [Manage files](https://developer.1password.com/docs/sdks/files)
- [Share items](https://developer.1password.com/docs/sdks/share-items)
- [List vaults and items](https://developer.1password.com/docs/sdks/list-vaults-items)
- [Manage vaults](https://developer.1password.com/docs/sdks/vaults)
- [Manage vault permissions](https://developer.1password.com/docs/sdks/vault-permissions)
- [Manage groups](https://developer.1password.com/docs/sdks/groups)
- [1Password SDK concepts](https://developer.1password.com/docs/sdks/concepts)

## License

Licensed under the [MIT License](LICENSE).
