# 1Password Rust SDK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the 1Password Go SDK (v0.4.1-beta.1) to idiomatic Rust with full API parity.

**Architecture:** Typed Rust wrapper around the same `core.wasm` binary used by Go/Python/JS SDKs. JSON-based RPC to the WASM core via Extism host SDK. Desktop app integration via `libloading` behind a `desktop` feature flag.

**Tech Stack:** Rust 2024 edition, extism, serde/serde_json, thiserror, libloading (desktop feature), getrandom, chrono

---

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` | Crate manifest with `desktop` feature flag |
| `rust-toolchain.toml` | Pin Rust 1.93.1, rustfmt, clippy |
| `deny.toml` | License/advisory/source auditing |
| `.gitignore` | Ignore target/, IDE files |
| `CLAUDE.md` | Developer conventions |
| `src/lib.rs` | Crate root, `#![deny(unsafe_code)]`, public re-exports |
| `src/errors.rs` | `SdkError` enum via thiserror |
| `src/core.rs` | `Core` trait + `InnerClient` + `InvokeConfig` structs |
| `src/core_extism.rs` | `ExtismCore` — WASM runtime + host functions |
| `src/core_shared_lib.rs` | `SharedLibCore` — native FFI (desktop feature, cfg unix/windows) |
| `src/client.rs` | `Client` struct + `ClientBuilder` |
| `src/types.rs` | All domain types (structs, enums, serde) |
| `src/secrets.rs` | `SecretsApi` trait + `SecretsSource` impl |
| `src/items.rs` | `ItemsApi` trait + `ItemsSource` impl |
| `src/items_files.rs` | `ItemsFilesApi` trait + impl |
| `src/items_shares.rs` | `ItemsSharesApi` trait + impl |
| `src/vaults.rs` | `VaultsApi` trait + `VaultsSource` impl |
| `src/groups.rs` | `GroupsApi` trait + `GroupsSource` impl |
| `src/environments.rs` | `EnvironmentsApi` trait + impl |
| `wasm/core.wasm` | Embedded WASM binary (~9.1MB) |
| `examples/service_account.rs` | Service account usage example |
| `examples/desktop_app.rs` | Desktop app usage example |
| `.github/workflows/ci.yml` | CI: fmt, clippy, deny, test |

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `deny.toml`
- Create: `.gitignore`
- Create: `CLAUDE.md`
- Create: `src/lib.rs`

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "onepassword-sdk"
version = "0.4.1-beta.1"
edition = "2024"
publish = false
license = "MIT"
description = "1Password SDK for Rust"

[features]
default = []
desktop = ["dep:libloading"]

[dependencies]
chrono = { version = "0.4", default-features = false, features = ["clock"] }
extism = "1"
getrandom = "0.2"
libloading = { version = "0.8", optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
```

- [ ] **Step 2: Create `rust-toolchain.toml`**

```toml
[toolchain]
channel = "1.93.1"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 3: Create `deny.toml`**

```toml
[graph]
all-features = true

[advisories]
unmaintained = "warn"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "OpenSSL",
    "Zlib",
]
confidence-threshold = 0.8

[[licenses.exceptions]]
allow = ["MPL-2.0"]
crate = "webpki-roots"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

- [ ] **Step 4: Create `.gitignore`**

```
/target
.DS_Store
.idea/
*.swp
```

- [ ] **Step 5: Create `CLAUDE.md`**

```markdown
# 1Password Rust SDK

## Build Commands

- `cargo fmt --check` — check formatting
- `cargo clippy -- -D warnings` — lint
- `cargo test` — run tests
- `cargo build` — build (default features)
- `cargo build --features desktop` — build with desktop app integration

## Conventions

- Edition 2024, Rust 1.93.1
- `#![deny(unsafe_code)]` at crate root; `#[allow(unsafe_code)]` only on FFI modules
- `thiserror` for error types
- `serde` for all JSON serialization
- Inline `#[cfg(test)] mod tests` blocks
- Conventional commits: `type(scope): description`
- Run `cargo fmt --check && cargo clippy -- -D warnings && cargo test` before pushing

## Architecture

Typed wrapper around an opaque WASM core binary. All API calls serialize params to JSON,
call `Core::invoke()`, and deserialize the response. Two core backends:
- `ExtismCore` (default) — embedded WASM via Extism
- `SharedLibCore` (feature: `desktop`) — native shared library from 1Password desktop app
```

- [ ] **Step 6: Create `src/lib.rs`**

```rust
#![deny(unsafe_code)]

mod client;
mod core;
mod core_extism;
mod errors;
mod types;

mod secrets;
mod items;
mod items_files;
mod items_shares;
mod vaults;
mod groups;
mod environments;

#[cfg(feature = "desktop")]
mod core_shared_lib;

pub use client::{Client, ClientBuilder};
pub use errors::SdkError;
pub use types::*;

pub use secrets::SecretsApi;
pub use items::ItemsApi;
pub use items_files::ItemsFilesApi;
pub use items_shares::ItemsSharesApi;
pub use vaults::VaultsApi;
pub use groups::GroupsApi;
pub use environments::EnvironmentsApi;
```

- [ ] **Step 7: Verify it compiles (will fail — stubs needed, but Cargo.toml is valid)**

Run: `cargo check 2>&1 | head -5`
Expected: Errors about missing modules (this is fine — we'll fill them in)

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml rust-toolchain.toml deny.toml .gitignore CLAUDE.md src/lib.rs
git commit -m "feat: scaffold project with Cargo.toml, toolchain, and lib.rs"
```

---

### Task 2: Error Types

**Files:**
- Create: `src/errors.rs`

- [ ] **Step 1: Write test for error deserialization**

Add to `src/errors.rs`:

```rust
use serde::Deserialize;
use serde_json;

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("1Password SDK error: {message}")]
    Core { name: String, message: String },

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("WASM plugin error: {0}")]
    Plugin(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("desktop session expired: {0}")]
    DesktopSessionExpired(String),

    #[error("rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("shared library error: {0}")]
    SharedLib(String),
}

#[derive(Deserialize)]
struct CoreError {
    name: String,
    message: String,
}

/// Deserialize a JSON error string from the WASM core into an SdkError.
/// Matches Go SDK's `unmarshalError` behavior.
pub(crate) fn unmarshal_error(err: &str) -> SdkError {
    match serde_json::from_str::<CoreError>(err) {
        Ok(core_err) => match core_err.name.as_str() {
            "DesktopSessionExpired" => SdkError::DesktopSessionExpired(core_err.message),
            "RateLimitExceeded" => SdkError::RateLimitExceeded(core_err.message),
            _ => SdkError::Core {
                name: core_err.name,
                message: core_err.message,
            },
        },
        Err(_) => SdkError::Core {
            name: "Unknown".to_string(),
            message: err.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarshal_core_error() {
        let err = unmarshal_error(r#"{"name":"SomeError","message":"something broke"}"#);
        match err {
            SdkError::Core { name, message } => {
                assert_eq!(name, "SomeError");
                assert_eq!(message, "something broke");
            }
            _ => panic!("expected Core error"),
        }
    }

    #[test]
    fn unmarshal_session_expired() {
        let err = unmarshal_error(r#"{"name":"DesktopSessionExpired","message":"session gone"}"#);
        assert!(matches!(err, SdkError::DesktopSessionExpired(_)));
    }

    #[test]
    fn unmarshal_rate_limit() {
        let err = unmarshal_error(r#"{"name":"RateLimitExceeded","message":"slow down"}"#);
        assert!(matches!(err, SdkError::RateLimitExceeded(_)));
    }

    #[test]
    fn unmarshal_invalid_json() {
        let err = unmarshal_error("not json at all");
        match err {
            SdkError::Core { message, .. } => assert_eq!(message, "not json at all"),
            _ => panic!("expected Core error for invalid JSON"),
        }
    }
}
```

- [ ] **Step 2: Verify tests pass**

Run: `cargo test -p onepassword-sdk errors::tests 2>&1`
Expected: Will fail until other modules exist. We'll verify after Task 3.

- [ ] **Step 3: Commit**

```bash
git add src/errors.rs
git commit -m "feat: add SdkError type and unmarshal_error"
```

---

### Task 3: Core Trait and Internal Types

**Files:**
- Create: `src/core.rs`

- [ ] **Step 1: Write the Core trait and supporting types**

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::SdkError;

pub(crate) const SDK_LANGUAGE: &str = "Rust";
pub(crate) const DEFAULT_REQUEST_LIBRARY: &str = "reqwest";
pub(crate) const MESSAGE_LIMIT: usize = 50 * 1024 * 1024;

pub(crate) trait Core: Send + Sync {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn release_client(&self, client_id: &[u8]);
}

#[derive(Debug, Serialize)]
pub(crate) struct ClientConfig {
    #[serde(rename = "serviceAccountToken")]
    pub sa_token: String,
    #[serde(rename = "programmingLanguage")]
    pub language: String,
    #[serde(rename = "sdkVersion")]
    pub sdk_version: String,
    #[serde(rename = "integrationName")]
    pub integration_name: String,
    #[serde(rename = "integrationVersion")]
    pub integration_version: String,
    #[serde(rename = "requestLibraryName")]
    pub request_library_name: String,
    #[serde(rename = "requestLibraryVersion")]
    pub request_library_version: String,
    #[serde(rename = "os")]
    pub system_os: String,
    #[serde(rename = "osVersion")]
    pub system_os_version: String,
    #[serde(rename = "architecture")]
    pub system_arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_name: Option<String>,
}

impl ClientConfig {
    pub fn new_default() -> Self {
        Self {
            sa_token: String::new(),
            language: SDK_LANGUAGE.to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
            integration_name: "Unknown".to_string(),
            integration_version: "Unknown".to_string(),
            request_library_name: DEFAULT_REQUEST_LIBRARY.to_string(),
            request_library_version: "0.0.0".to_string(),
            system_os: std::env::consts::OS.to_string(),
            system_os_version: "0.0.0".to_string(),
            system_arch: std::env::consts::ARCH.to_string(),
            account_name: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct InvokeConfig {
    pub invocation: Invocation,
}

#[derive(Debug, Serialize)]
pub(crate) struct Invocation {
    #[serde(rename = "clientId", skip_serializing_if = "Option::is_none")]
    pub client_id: Option<u64>,
    pub parameters: Parameters,
}

#[derive(Debug, Serialize)]
pub(crate) struct Parameters {
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Wraps a Core implementation and handles JSON marshaling/unmarshaling.
pub(crate) struct CoreWrapper {
    pub inner: Box<dyn Core>,
}

impl CoreWrapper {
    pub fn init_client(&self, config: &ClientConfig) -> Result<u64, SdkError> {
        let config_bytes = serde_json::to_vec(config)?;
        let res = self.inner.init_client(&config_bytes)?;
        let id: u64 = serde_json::from_slice(&res)?;
        Ok(id)
    }

    pub fn invoke(&self, invoke_config: &InvokeConfig) -> Result<String, SdkError> {
        let input = serde_json::to_vec(invoke_config)?;
        if input.len() > MESSAGE_LIMIT {
            return Err(SdkError::Config(format!(
                "message size exceeds the limit of {} bytes",
                MESSAGE_LIMIT
            )));
        }
        let res = self.inner.invoke(&input)?;
        Ok(String::from_utf8_lossy(&res).into_owned())
    }

    pub fn release_client(&self, client_id: u64) {
        if let Ok(id_bytes) = serde_json::to_vec(&client_id) {
            self.inner.release_client(&id_bytes);
        }
    }
}

/// The inner client state shared by all API implementations.
pub(crate) struct InnerClient {
    pub id: u64,
    pub config: ClientConfig,
    pub core: CoreWrapper,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_language() {
        let config = ClientConfig::new_default();
        assert_eq!(config.language, "Rust");
        assert_eq!(config.system_os, std::env::consts::OS);
        assert_eq!(config.system_arch, std::env::consts::ARCH);
    }

    #[test]
    fn invoke_config_serializes_correctly() {
        let config = InvokeConfig {
            invocation: Invocation {
                client_id: Some(42),
                parameters: Parameters {
                    name: "SecretsResolve".to_string(),
                    parameters: {
                        let mut m = HashMap::new();
                        m.insert(
                            "secret_reference".to_string(),
                            serde_json::Value::String("op://vault/item/field".to_string()),
                        );
                        m
                    },
                },
            },
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"clientId\":42"));
        assert!(json.contains("\"name\":\"SecretsResolve\""));
    }
}
```

- [ ] **Step 2: Verify tests pass**

Run: `cargo test core::tests`
Expected: PASS (2 tests)

- [ ] **Step 3: Commit**

```bash
git add src/core.rs
git commit -m "feat: add Core trait, ClientConfig, InvokeConfig, CoreWrapper"
```

---

### Task 4: Domain Types

**Files:**
- Create: `src/types.rs`

This is the largest file — all domain types ported from Go's `types.go`. The Go version uses manual JSON marshal/unmarshal for tagged unions; Rust uses native `#[serde(tag, content)]` enums.

- [ ] **Step 1: Write all simple struct types**

Create `src/types.rs` with all struct types, string enums, and constants. The full content is:

```rust
use serde::{Deserialize, Serialize};

// --- Error message type ---

pub type ErrorMessage = String;

// --- Address ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressFieldDetails {
    pub street: String,
    pub city: String,
    pub country: String,
    pub zip: String,
    pub state: String,
}

// --- Documents & Files ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreateParams {
    pub name: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttributes {
    pub name: String,
    pub id: String,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCreateParams {
    pub name: String,
    pub content: Vec<u8>,
    pub section_id: String,
    pub field_id: String,
}

// --- Environments ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
    pub masked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariablesResponse {
    pub variables: Vec<EnvironmentVariable>,
}

// --- Password generation ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePasswordResponse {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SeparatorType {
    Digits,
    DigitsAndSymbols,
    Spaces,
    Hyphens,
    Underscores,
    Periods,
    Commas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WordListType {
    FullWords,
    Syllables,
    ThreeLetters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordRecipeMemorableInner {
    pub separator_type: SeparatorType,
    pub capitalize: bool,
    pub word_list_type: WordListType,
    pub word_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRecipePinInner {
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordRecipeRandomInner {
    pub include_digits: bool,
    pub include_symbols: bool,
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters")]
pub enum PasswordRecipe {
    Memorable(PasswordRecipeMemorableInner),
    Pin(PasswordRecipePinInner),
    Random(PasswordRecipeRandomInner),
}

// --- SSH Keys ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHKeyAttributes {
    pub public_key: String,
    pub fingerprint: String,
    pub key_type: String,
}

// --- OTP ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OTPFieldDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

// --- Item field details (tagged enum) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ItemFieldDetails {
    Otp(OTPFieldDetails),
    SshKey(SSHKeyAttributes),
    Address(AddressFieldDetails),
}

// --- Item categories ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemCategory {
    Login,
    SecureNote,
    CreditCard,
    CryptoWallet,
    Identity,
    Password,
    Document,
    ApiCredentials,
    BankAccount,
    Database,
    DriverLicense,
    Email,
    MedicalRecord,
    Membership,
    OutdoorLicense,
    Passport,
    Rewards,
    Router,
    Server,
    SshKey,
    SocialSecurityNumber,
    SoftwareLicense,
    Person,
    Unsupported,
}

// --- Item field types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemFieldType {
    Text,
    Concealed,
    CreditCardType,
    CreditCardNumber,
    Phone,
    Url,
    Totp,
    Email,
    Reference,
    SshKey,
    Menu,
    MonthYear,
    Address,
    Date,
    Unsupported,
}

// --- Item state ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemState {
    Active,
    Archived,
}

// --- Autofill ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutofillBehavior {
    AnywhereOnWebsite,
    ExactDomain,
    Never,
}

// --- Website ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Website {
    pub url: String,
    pub label: String,
    pub autofill_behavior: AutofillBehavior,
}

// --- Item field ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemField {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id: Option<String>,
    pub field_type: ItemFieldType,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ItemFieldDetails>,
}

// --- Item section ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSection {
    pub id: String,
    pub title: String,
}

// --- Item file ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFile {
    pub attributes: FileAttributes,
    pub section_id: String,
    pub field_id: String,
}

// --- Item ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub title: String,
    pub category: ItemCategory,
    pub vault_id: String,
    pub fields: Vec<ItemField>,
    pub sections: Vec<ItemSection>,
    pub notes: String,
    pub tags: Vec<String>,
    pub websites: Vec<Website>,
    pub version: u32,
    pub files: Vec<ItemFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<FileAttributes>,
    pub created_at: String,
    pub updated_at: String,
}

// --- Item create params ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCreateParams {
    pub category: ItemCategory,
    pub vault_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<ItemField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<ItemSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websites: Option<Vec<Website>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileCreateParams>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentCreateParams>,
}

// --- Item overview ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemOverview {
    pub id: String,
    pub title: String,
    pub category: ItemCategory,
    pub vault_id: String,
    pub websites: Vec<Website>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub state: ItemState,
}

// --- Item list filter ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemListFilterByStateInner {
    pub active: bool,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ItemListFilter {
    ByState(ItemListFilterByStateInner),
}

// --- Item sharing ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemShareDuration {
    OneHour,
    OneDay,
    SevenDays,
    FourteenDays,
    ThirtyDays,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedType {
    Authenticated,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedRecipientType {
    Email,
    Domain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemShareFiles {
    pub allowed: bool,
    pub max_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_types: Option<Vec<AllowedType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_recipient_types: Option<Vec<AllowedRecipientType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_expiry: Option<ItemShareDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_expiry: Option<ItemShareDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_views: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemShareAccountPolicy {
    pub max_expiry: ItemShareDuration,
    pub default_expiry: ItemShareDuration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_views: Option<u32>,
    pub allowed_types: Vec<AllowedType>,
    pub allowed_recipient_types: Vec<AllowedRecipientType>,
    pub files: ItemShareFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidRecipientEmailInner {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidRecipientDomainInner {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters")]
pub enum ValidRecipient {
    Email(ValidRecipientEmailInner),
    Domain(ValidRecipientDomainInner),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemShareParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<ValidRecipient>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_after: Option<ItemShareDuration>,
    pub one_time_only: bool,
}

// --- Batch response types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse<T, E> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

// --- Item update failure reasons ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ItemUpdateFailureReason {
    ItemValidationError(ErrorMessage),
    ItemStatusPermissionError,
    ItemStatusIncorrectItemVersion,
    ItemStatusFileNotFound,
    ItemStatusTooBig,
    ItemNotFound,
    Internal(ErrorMessage),
}

// --- Items batch responses ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsDeleteAllResponse {
    pub individual_responses:
        std::collections::HashMap<String, BatchResponse<(), ItemUpdateFailureReason>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemsGetAllError {
    #[serde(rename = "itemNotFound")]
    ItemNotFound,
    #[serde(rename = "internal")]
    Internal(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsGetAllResponse {
    pub individual_responses: Vec<BatchResponse<Item, ItemsGetAllError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsUpdateAllResponse {
    pub individual_responses: Vec<BatchResponse<Item, ItemUpdateFailureReason>>,
}

// --- Secrets ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedReference {
    pub secret: String,
    pub item_id: String,
    pub vault_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ResolveReferenceError {
    Parsing(ErrorMessage),
    FieldNotFound,
    VaultNotFound,
    TooManyVaults,
    ItemNotFound,
    TooManyItems,
    TooManyMatchingFields,
    NoMatchingSections,
    IncompatibleTOTPQueryParameterField,
    #[serde(rename = "unableToGenerateTotpCode")]
    UnableToGenerateTOTPCode(ErrorMessage),
    #[serde(rename = "sSHKeyMetadataNotFound")]
    SSHKeyMetadataNotFound,
    UnsupportedFileFormat,
    #[serde(rename = "incompatibleSshKeyQueryParameterField")]
    IncompatibleSSHKeyQueryParameterField,
    UnableToParsePrivateKey,
    #[serde(rename = "unableToFormatPrivateKeyToOpenSsh")]
    UnableToFormatPrivateKeyToOpenSSH,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveAllResponse {
    pub individual_responses:
        std::collections::HashMap<String, BatchResponse<ResolvedReference, ResolveReferenceError>>,
}

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupType {
    Owners,
    Administrators,
    Recovery,
    ExternalAccountManagers,
    TeamMembers,
    UserDefined,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupState {
    Active,
    Deleted,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VaultAccessorType {
    User,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultAccess {
    pub vault_uuid: String,
    pub accessor_type: VaultAccessorType,
    pub accessor_uuid: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub title: String,
    pub description: String,
    pub group_type: GroupType,
    pub state: GroupState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_access: Option<Vec<VaultAccess>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAccess {
    pub group_id: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGetParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_permissions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupVaultAccess {
    pub vault_id: String,
    pub group_id: String,
    pub permissions: u32,
}

// --- Vaults ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VaultType {
    Personal,
    Everyone,
    Transfer,
    UserCreated,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    pub id: String,
    pub title: String,
    pub description: String,
    pub vault_type: VaultType,
    pub active_item_count: u32,
    pub content_version: u32,
    pub attribute_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<Vec<VaultAccess>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCreateParams {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_admins_access: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultGetParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessors: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decrypt_details: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultOverview {
    pub id: String,
    pub title: String,
    pub description: String,
    pub vault_type: VaultType,
    pub active_item_count: u32,
    pub content_version: u32,
    pub attribute_version: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultUpdateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// --- Permissions constants ---

pub mod permissions {
    pub const READ_ITEMS: u32 = 32;
    pub const REVEAL_ITEM_PASSWORD: u32 = 16;
    pub const UPDATE_ITEMS: u32 = 64;
    pub const CREATE_ITEMS: u32 = 128;
    pub const ARCHIVE_ITEMS: u32 = 256;
    pub const DELETE_ITEMS: u32 = 512;
    pub const UPDATE_ITEM_HISTORY: u32 = 1024;
    pub const SEND_ITEMS: u32 = 1_048_576;
    pub const IMPORT_ITEMS: u32 = 2_097_152;
    pub const EXPORT_ITEMS: u32 = 4_194_304;
    pub const PRINT_ITEMS: u32 = 8_388_608;
    pub const MANAGE_VAULT: u32 = 2;
    pub const RECOVER_VAULT: u32 = 1;
    pub const NO_ACCESS: u32 = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_recipe_serializes_as_tagged_enum() {
        let recipe = PasswordRecipe::Random(PasswordRecipeRandomInner {
            include_digits: true,
            include_symbols: false,
            length: 20,
        });
        let json = serde_json::to_string(&recipe).unwrap();
        assert!(json.contains(r#""type":"Random"#));
        assert!(json.contains(r#""parameters":#));
    }

    #[test]
    fn item_field_details_roundtrip() {
        let details = ItemFieldDetails::Otp(OTPFieldDetails {
            code: Some("123456".to_string()),
            error_message: None,
        });
        let json = serde_json::to_string(&details).unwrap();
        let parsed: ItemFieldDetails = serde_json::from_str(&json).unwrap();
        match parsed {
            ItemFieldDetails::Otp(otp) => assert_eq!(otp.code.unwrap(), "123456"),
            _ => panic!("expected Otp variant"),
        }
    }

    #[test]
    fn valid_recipient_roundtrip() {
        let recipient = ValidRecipient::Email(ValidRecipientEmailInner {
            email: "test@example.com".to_string(),
        });
        let json = serde_json::to_string(&recipient).unwrap();
        assert!(json.contains(r#""type":"Email"#));
        let parsed: ValidRecipient = serde_json::from_str(&json).unwrap();
        match parsed {
            ValidRecipient::Email(inner) => assert_eq!(inner.email, "test@example.com"),
            _ => panic!("expected Email variant"),
        }
    }

    #[test]
    fn item_list_filter_roundtrip() {
        let filter = ItemListFilter::ByState(ItemListFilterByStateInner {
            active: true,
            archived: false,
        });
        let json = serde_json::to_string(&filter).unwrap();
        let parsed: ItemListFilter = serde_json::from_str(&json).unwrap();
        match parsed {
            ItemListFilter::ByState(inner) => {
                assert!(inner.active);
                assert!(!inner.archived);
            }
        }
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test types::tests`
Expected: PASS (4 tests)

- [ ] **Step 3: Commit**

```bash
git add src/types.rs
git commit -m "feat: add all domain types with serde support"
```

---

### Task 5: ExtismCore (WASM Runtime)

**Files:**
- Create: `src/core_extism.rs`
- Copy: `wasm/core.wasm` from Go SDK

- [ ] **Step 1: Copy the WASM binary**

```bash
mkdir -p wasm
cp /tmp/onepassword-sdk-go/internal/wasm/core.wasm wasm/core.wasm
```

- [ ] **Step 2: Write `src/core_extism.rs`**

```rust
use std::sync::Mutex;

use extism::{host_fn, Manifest, Plugin, PluginBuilder, Wasm};

use crate::core::{Core, MESSAGE_LIMIT};
use crate::errors::SdkError;

static CORE_WASM: &[u8] = include_bytes!("../wasm/core.wasm");

const INVOKE_FUNC: &str = "invoke";
const INIT_CLIENT_FUNC: &str = "init_client";
const RELEASE_CLIENT_FUNC: &str = "release_client";

pub(crate) struct ExtismCore {
    plugin: Mutex<Plugin>,
}

impl ExtismCore {
    pub fn new() -> Result<Self, SdkError> {
        let manifest = Manifest::new([Wasm::data(CORE_WASM)])
            .with_allowed_hosts(allowed_1p_hosts().into_iter());

        let plugin = PluginBuilder::new(manifest)
            .with_wasi(true)
            .build()
            .map_err(|e| SdkError::Plugin(format!("failed to initialize plugin: {e}")))?;

        Ok(Self {
            plugin: Mutex::new(plugin),
        })
    }
}

impl Core for ExtismCore {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError> {
        let mut plugin = self.plugin.lock().unwrap();
        let res = plugin
            .call::<&[u8], Vec<u8>>(INIT_CLIENT_FUNC, config)
            .map_err(|e| SdkError::Plugin(e.to_string()))?;
        Ok(res)
    }

    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
        if invoke_config.len() > MESSAGE_LIMIT {
            return Err(SdkError::Config(format!(
                "message size exceeds the limit of {} bytes",
                MESSAGE_LIMIT
            )));
        }
        let mut plugin = self.plugin.lock().unwrap();
        let res = plugin
            .call::<&[u8], Vec<u8>>(INVOKE_FUNC, invoke_config)
            .map_err(|e| SdkError::Plugin(e.to_string()))?;
        Ok(res)
    }

    fn release_client(&self, client_id: &[u8]) {
        let mut plugin = self.plugin.lock().unwrap();
        let _ = plugin.call::<&[u8], Vec<u8>>(RELEASE_CLIENT_FUNC, client_id);
    }
}

fn allowed_1p_hosts() -> Vec<String> {
    [
        "*.1password.com",
        "*.1password.ca",
        "*.1password.eu",
        "*.b5staging.com",
        "*.b5dev.com",
        "*.b5dev.ca",
        "*.b5dev.eu",
        "*.b5test.com",
        "*.b5test.ca",
        "*.b5test.eu",
        "*.b5rev.com",
        "*.b5local.com",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
```

**Note:** The Extism Rust SDK's API may differ from this sketch. The implementer must check the `extism` crate docs (use context7 or `cargo doc`) for the actual `PluginBuilder`/`Manifest`/`Plugin::call` API signatures. The Go SDK registers host functions for `random_fill_imported` (namespace `op-extism-core`), `unix_time_milliseconds_imported` (namespaces `op-now` and `zxcvbn`), and `utc_offset_seconds` (namespace `op-time`). These must be registered when building the plugin. The exact Rust API for host functions with namespaces needs to be looked up. If the Extism Rust SDK doesn't support namespaced host functions in the same way, this will need adaptation.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: Should compile (or reveal Extism API differences to fix)

- [ ] **Step 4: Commit**

```bash
git add wasm/core.wasm src/core_extism.rs
git commit -m "feat: add ExtismCore WASM runtime with embedded core.wasm"
```

---

### Task 6: Client Builder

**Files:**
- Create: `src/client.rs`

- [ ] **Step 1: Write the Client and ClientBuilder**

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{ClientConfig, CoreWrapper, InnerClient, InvokeConfig, Invocation, Parameters};
use crate::core_extism::ExtismCore;
use crate::errors::{unmarshal_error, SdkError};
use crate::environments::{EnvironmentsApi, EnvironmentsSource};
use crate::groups::{GroupsApi, GroupsSource};
use crate::items::{ItemsApi, ItemsSource};
use crate::secrets::{SecretsApi, SecretsSource};
use crate::vaults::{VaultsApi, VaultsSource};

pub struct Client {
    inner: Arc<InnerClient>,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            config: ClientConfig::new_default(),
        }
    }

    pub fn secrets(&self) -> impl SecretsApi + '_ {
        SecretsSource::new(&self.inner)
    }

    pub fn items(&self) -> impl ItemsApi + '_ {
        ItemsSource::new(&self.inner)
    }

    pub fn vaults(&self) -> impl VaultsApi + '_ {
        VaultsSource::new(&self.inner)
    }

    pub fn groups(&self) -> impl GroupsApi + '_ {
        GroupsSource::new(&self.inner)
    }

    pub fn environments(&self) -> impl EnvironmentsApi + '_ {
        EnvironmentsSource::new(&self.inner)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.inner.core.release_client(self.inner.id);
    }
}

pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn service_account_token(mut self, token: &str) -> Self {
        self.config.sa_token = token.to_string();
        self
    }

    #[cfg(feature = "desktop")]
    pub fn desktop_app_integration(mut self, account_name: &str) -> Self {
        self.config.account_name = Some(account_name.to_string());
        self
    }

    pub fn integration_info(mut self, name: &str, version: &str) -> Self {
        self.config.integration_name = name.to_string();
        self.config.integration_version = version.to_string();
        self
    }

    pub fn build(self) -> Result<Client, SdkError> {
        let has_sa_token = !self.config.sa_token.is_empty();
        let has_desktop = self.config.account_name.is_some();

        if has_sa_token && has_desktop {
            return Err(SdkError::Config(
                "cannot use both SA token and desktop app authentication".to_string(),
            ));
        }

        let core_impl: Box<dyn crate::core::Core> = if has_desktop {
            #[cfg(feature = "desktop")]
            {
                let account_name = self.config.account_name.as_ref().unwrap().clone();
                Box::new(crate::core_shared_lib::SharedLibCore::new(&account_name)?)
            }
            #[cfg(not(feature = "desktop"))]
            {
                return Err(SdkError::Config(
                    "desktop app integration requires the 'desktop' feature".to_string(),
                ));
            }
        } else {
            Box::new(ExtismCore::new()?)
        };

        let core = CoreWrapper { inner: core_impl };

        let client_id = core
            .init_client(&self.config)
            .map_err(|e| SdkError::Config(format!("error initializing client: {e}")))?;

        let inner = Arc::new(InnerClient {
            id: client_id,
            config: self.config,
            core,
        });

        Ok(Client { inner })
    }
}

/// Invoke a method on the WASM core, with automatic retry on desktop session expiry.
pub(crate) fn client_invoke(
    inner: &InnerClient,
    method: &str,
    params: HashMap<String, serde_json::Value>,
) -> Result<String, SdkError> {
    let invoke_config = InvokeConfig {
        invocation: Invocation {
            client_id: Some(inner.id),
            parameters: Parameters {
                name: method.to_string(),
                parameters: params,
            },
        },
    };

    match inner.core.invoke(&invoke_config) {
        Ok(response) => Ok(response),
        Err(e) => Err(unmarshal_error(&e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_rejects_both_auth_methods() {
        let mut builder = Client::builder()
            .service_account_token("ops_test");
        builder.config.account_name = Some("myaccount".to_string());
        let result = builder.build();
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cannot use both"));
    }
}
```

- [ ] **Step 2: Run test**

Run: `cargo test client::tests`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/client.rs
git commit -m "feat: add Client struct and ClientBuilder with auth validation"
```

---

### Task 7: API Implementations — Secrets

**Files:**
- Create: `src/secrets.rs`

- [ ] **Step 1: Write SecretsApi trait and implementation**

```rust
use std::collections::HashMap;

use crate::client::client_invoke;
use crate::core::InnerClient;
use crate::errors::SdkError;
use crate::types::{GeneratePasswordResponse, PasswordRecipe, ResolveAllResponse};

pub trait SecretsApi {
    fn resolve(&self, secret_reference: &str) -> Result<String, SdkError>;
    fn resolve_all(&self, secret_references: &[String]) -> Result<ResolveAllResponse, SdkError>;
}

pub(crate) struct SecretsSource<'a> {
    inner: &'a InnerClient,
}

impl<'a> SecretsSource<'a> {
    pub fn new(inner: &'a InnerClient) -> Self {
        Self { inner }
    }
}

impl SecretsApi for SecretsSource<'_> {
    fn resolve(&self, secret_reference: &str) -> Result<String, SdkError> {
        let mut params = HashMap::new();
        params.insert(
            "secret_reference".to_string(),
            serde_json::Value::String(secret_reference.to_string()),
        );
        let result_string = client_invoke(self.inner, "SecretsResolve", params)?;
        let result: String = serde_json::from_str(&result_string)?;
        Ok(result)
    }

    fn resolve_all(&self, secret_references: &[String]) -> Result<ResolveAllResponse, SdkError> {
        let mut params = HashMap::new();
        params.insert(
            "secret_references".to_string(),
            serde_json::to_value(secret_references)?,
        );
        let result_string = client_invoke(self.inner, "SecretsResolveAll", params)?;
        let result: ResolveAllResponse = serde_json::from_str(&result_string)?;
        Ok(result)
    }
}

/// Standalone secret utilities (no client required).
pub struct Secrets;

impl Secrets {
    pub fn validate_secret_reference(secret_reference: &str) -> Result<(), SdkError> {
        let core = crate::core_extism::ExtismCore::new()?;
        let core_wrapper = crate::core::CoreWrapper {
            inner: Box::new(core),
        };
        let invoke_config = crate::core::InvokeConfig {
            invocation: crate::core::Invocation {
                client_id: None,
                parameters: crate::core::Parameters {
                    name: "ValidateSecretReference".to_string(),
                    parameters: {
                        let mut m = HashMap::new();
                        m.insert(
                            "secret_reference".to_string(),
                            serde_json::Value::String(secret_reference.to_string()),
                        );
                        m
                    },
                },
            },
        };
        core_wrapper.invoke(&invoke_config)?;
        Ok(())
    }

    pub fn generate_password(
        recipe: PasswordRecipe,
    ) -> Result<GeneratePasswordResponse, SdkError> {
        let core = crate::core_extism::ExtismCore::new()?;
        let core_wrapper = crate::core::CoreWrapper {
            inner: Box::new(core),
        };
        let invoke_config = crate::core::InvokeConfig {
            invocation: crate::core::Invocation {
                client_id: None,
                parameters: crate::core::Parameters {
                    name: "GeneratePassword".to_string(),
                    parameters: {
                        let mut m = HashMap::new();
                        m.insert("recipe".to_string(), serde_json::to_value(&recipe)?);
                        m
                    },
                },
            },
        };
        let result_string = core_wrapper.invoke(&invoke_config)?;
        let result: GeneratePasswordResponse = serde_json::from_str(&result_string)?;
        Ok(result)
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/secrets.rs
git commit -m "feat: add SecretsApi with resolve, resolve_all, and standalone utils"
```

---

### Task 8: API Implementations — Items, ItemsFiles, ItemsShares

**Files:**
- Create: `src/items.rs`
- Create: `src/items_files.rs`
- Create: `src/items_shares.rs`

- [ ] **Step 1: Write `src/items.rs`**

```rust
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
    fn get_all(
        &self,
        vault_id: &str,
        item_ids: &[String],
    ) -> Result<ItemsGetAllResponse, SdkError>;
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
        let result_string = client_invoke(self.inner, "ItemsCreate", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsCreateAll", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsGet", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsGetAll", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn put(&self, item: Item) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        let result_string = client_invoke(self.inner, "ItemsPut", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsDeleteAll", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsList", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn shares(&self) -> impl ItemsSharesApi + '_ {
        ItemsSharesSource::new(self.inner)
    }

    fn files(&self) -> impl ItemsFilesApi + '_ {
        ItemsFilesSource::new(self.inner)
    }
}
```

- [ ] **Step 2: Write `src/items_files.rs`**

```rust
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
        p.insert("file_params".to_string(), serde_json::to_value(&file_params)?);
        let result_string = client_invoke(self.inner, "ItemsFilesAttach", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsFilesRead", p)?;
        let result: Vec<u8> = serde_json::from_str(&result_string)?;
        Ok(result)
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
        let result_string = client_invoke(self.inner, "ItemsFilesDelete", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn replace_document(
        &self,
        item: Item,
        doc_params: DocumentCreateParams,
    ) -> Result<Item, SdkError> {
        let mut p = HashMap::new();
        p.insert("item".to_string(), serde_json::to_value(&item)?);
        p.insert("doc_params".to_string(), serde_json::to_value(&doc_params)?);
        let result_string = client_invoke(self.inner, "ItemsFilesReplaceDocument", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }
}
```

- [ ] **Step 3: Write `src/items_shares.rs`**

```rust
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
        let result_string = client_invoke(self.inner, "ItemsSharesGetAccountPolicy", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn validate_recipients(
        &self,
        policy: ItemShareAccountPolicy,
        recipients: &[String],
    ) -> Result<Vec<ValidRecipient>, SdkError> {
        let mut p = HashMap::new();
        p.insert("policy".to_string(), serde_json::to_value(&policy)?);
        p.insert("recipients".to_string(), serde_json::to_value(recipients)?);
        let result_string =
            client_invoke(self.inner, "ItemsSharesValidateRecipients", p)?;
        Ok(serde_json::from_str(&result_string)?)
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
        let result_string = client_invoke(self.inner, "ItemsSharesCreate", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add src/items.rs src/items_files.rs src/items_shares.rs
git commit -m "feat: add ItemsApi, ItemsFilesApi, and ItemsSharesApi"
```

---

### Task 9: API Implementations — Vaults, Groups, Environments

**Files:**
- Create: `src/vaults.rs`
- Create: `src/groups.rs`
- Create: `src/environments.rs`

- [ ] **Step 1: Write `src/vaults.rs`**

```rust
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
    fn revoke_group_permissions(
        &self,
        vault_id: &str,
        group_id: &str,
    ) -> Result<(), SdkError>;
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
        let result_string = client_invoke(self.inner, "VaultsCreate", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn list(&self, params: Option<VaultListParams>) -> Result<Vec<VaultOverview>, SdkError> {
        let mut p = HashMap::new();
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result_string = client_invoke(self.inner, "VaultsList", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn get_overview(&self, vault_id: &str) -> Result<VaultOverview, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        let result_string = client_invoke(self.inner, "VaultsGetOverview", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn get(&self, vault_id: &str, params: VaultGetParams) -> Result<Vault, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("vault_params".to_string(), serde_json::to_value(&params)?);
        let result_string = client_invoke(self.inner, "VaultsGet", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }

    fn update(&self, vault_id: &str, params: VaultUpdateParams) -> Result<Vault, SdkError> {
        let mut p = HashMap::new();
        p.insert(
            "vault_id".to_string(),
            serde_json::Value::String(vault_id.to_string()),
        );
        p.insert("params".to_string(), serde_json::to_value(&params)?);
        let result_string = client_invoke(self.inner, "VaultsUpdate", p)?;
        Ok(serde_json::from_str(&result_string)?)
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

    fn revoke_group_permissions(
        &self,
        vault_id: &str,
        group_id: &str,
    ) -> Result<(), SdkError> {
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
```

- [ ] **Step 2: Write `src/groups.rs`**

```rust
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
        let result_string = client_invoke(self.inner, "GroupsGet", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }
}
```

- [ ] **Step 3: Write `src/environments.rs`**

```rust
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
        let result_string = client_invoke(self.inner, "EnvironmentsGetVariables", p)?;
        Ok(serde_json::from_str(&result_string)?)
    }
}
```

- [ ] **Step 4: Verify full crate compiles**

Run: `cargo check`
Expected: PASS (no desktop feature, so shared_lib module is excluded)

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/vaults.rs src/groups.rs src/environments.rs
git commit -m "feat: add VaultsApi, GroupsApi, and EnvironmentsApi"
```

---

### Task 10: SharedLibCore (Desktop App Integration)

**Files:**
- Create: `src/core_shared_lib.rs`

- [ ] **Step 1: Write `src/core_shared_lib.rs`**

This file uses `#[allow(unsafe_code)]` for FFI operations. It uses `libloading` for cross-platform dlopen/LoadLibrary.

```rust
#![allow(unsafe_code)]

use std::ffi::c_void;
use std::path::PathBuf;

use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use crate::core::Core;
use crate::errors::SdkError;

#[derive(Serialize)]
struct Request<'a> {
    kind: &'a str,
    account_name: &'a str,
    payload: &'a [u8],
}

#[derive(Deserialize)]
struct Response {
    success: bool,
    payload: Vec<u8>,
}

type SendMessageFn = unsafe extern "C" fn(
    msg_ptr: *const u8,
    msg_len: usize,
    out_buf: *mut *mut u8,
    out_len: *mut usize,
    out_cap: *mut usize,
) -> i32;

type FreeResponseFn = unsafe extern "C" fn(buf: *mut u8, len: usize, cap: usize);

pub(crate) struct SharedLibCore {
    account_name: String,
    _library: Library,
    send_message: SendMessageFn,
    free_response: FreeResponseFn,
}

// SAFETY: The shared library functions are thread-safe according to 1Password's documentation.
// The Go SDK also uses them across goroutines.
unsafe impl Send for SharedLibCore {}
unsafe impl Sync for SharedLibCore {}

impl SharedLibCore {
    pub fn new(account_name: &str) -> Result<Self, SdkError> {
        let lib_path = find_1password_lib_path()?;

        let library = unsafe {
            Library::new(&lib_path)
                .map_err(|e| SdkError::SharedLib(format!("failed to open library: {e}")))?
        };

        let send_message: SendMessageFn = unsafe {
            let sym: Symbol<SendMessageFn> = library
                .get(b"op_sdk_ipc_send_message")
                .map_err(|e| SdkError::SharedLib(format!("failed to load send_message: {e}")))?;
            *sym
        };

        let free_response: FreeResponseFn = unsafe {
            let sym: Symbol<FreeResponseFn> = library
                .get(b"op_sdk_ipc_free_response")
                .map_err(|e| SdkError::SharedLib(format!("failed to load free_message: {e}")))?;
            *sym
        };

        Ok(Self {
            account_name: account_name.to_string(),
            _library: library,
            send_message,
            free_response,
        })
    }

    fn call_shared_library(&self, kind: &str, payload: &[u8]) -> Result<Vec<u8>, SdkError> {
        let request = Request {
            kind,
            account_name: &self.account_name,
            payload,
        };
        let input = serde_json::to_vec(&request)?;

        let mut out_buf: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let mut out_cap: usize = 0;

        let ret_code = unsafe {
            (self.send_message)(
                input.as_ptr(),
                input.len(),
                &mut out_buf,
                &mut out_len,
                &mut out_cap,
            )
        };

        error_from_return_code(ret_code)?;

        let resp_bytes = unsafe { std::slice::from_raw_parts(out_buf, out_len).to_vec() };

        unsafe {
            (self.free_response)(out_buf, out_len, out_cap);
        }

        let response: Response = serde_json::from_slice(&resp_bytes)
            .map_err(|e| SdkError::SharedLib(format!("failed to parse response: {e}")))?;

        if response.success {
            Ok(response.payload)
        } else {
            Err(SdkError::SharedLib(
                String::from_utf8_lossy(&response.payload).into_owned(),
            ))
        }
    }
}

impl Core for SharedLibCore {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError> {
        self.call_shared_library("init_client", config)
    }

    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
        self.call_shared_library("invoke", invoke_config)
    }

    fn release_client(&self, client_id: &[u8]) {
        let _ = self.call_shared_library("release_client", client_id);
    }
}

fn find_1password_lib_path() -> Result<PathBuf, SdkError> {
    let home = dirs_next::home_dir()
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .unwrap_or_default();

    let locations: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/Applications/1Password.app/Contents/Frameworks/libop_sdk_ipc_client.dylib"),
            home.join("Applications/1Password.app/Contents/Frameworks/libop_sdk_ipc_client.dylib"),
        ]
    } else if cfg!(target_os = "linux") {
        vec![
            PathBuf::from("/usr/bin/1password/libop_sdk_ipc_client.so"),
            PathBuf::from("/opt/1Password/libop_sdk_ipc_client.so"),
            PathBuf::from("/snap/bin/1password/libop_sdk_ipc_client.so"),
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            home.join(r"AppData\Local\1Password\op_sdk_ipc_client.dll"),
            PathBuf::from(r"C:\Program Files\1Password\app\8\op_sdk_ipc_client.dll"),
            PathBuf::from(r"C:\Program Files (x86)\1Password\app\8\op_sdk_ipc_client.dll"),
            home.join(r"AppData\Local\1Password\app\8\op_sdk_ipc_client.dll"),
        ]
    } else {
        return Err(SdkError::SharedLib(format!(
            "unsupported OS: {}",
            std::env::consts::OS
        )));
    };

    for path in &locations {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(SdkError::SharedLib(
        "1Password desktop application not found".to_string(),
    ))
}

fn error_from_return_code(ret_code: i32) -> Result<(), SdkError> {
    if ret_code == 0 {
        return Ok(());
    }

    let msg = if cfg!(target_os = "macos") {
        match ret_code {
            -3 => "desktop app connection channel is closed. Make sure Settings > Developer > Integrate with other apps is enabled".to_string(),
            -7 => "connection was unexpectedly dropped by the desktop app. Make sure the desktop app is running".to_string(),
            _ => format!("an internal error occurred. Return code: {ret_code}"),
        }
    } else {
        match ret_code {
            -2 => "desktop app connection channel is closed. Make sure Settings > Developer > Integrate with other apps is enabled".to_string(),
            -5 => "connection was unexpectedly dropped by the desktop app. Make sure the desktop app is running".to_string(),
            _ => format!("an internal error occurred. Return code: {ret_code}"),
        }
    };

    Err(SdkError::SharedLib(msg))
}
```

**Note:** The `dirs_next` crate is used for `home_dir()`. Add it to `Cargo.toml`:

```toml
[dependencies]
# ... existing deps ...
dirs-next = { version = "2", optional = true }

[features]
desktop = ["dep:libloading", "dep:dirs-next"]
```

Alternatively, use `std::env::var("HOME")` directly and skip the extra dep. The implementer should decide.

- [ ] **Step 2: Verify it compiles with the desktop feature**

Run: `cargo check --features desktop`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/core_shared_lib.rs
git commit -m "feat: add SharedLibCore for desktop app integration"
```

---

### Task 11: Examples

**Files:**
- Create: `examples/service_account.rs`
- Create: `examples/desktop_app.rs`

- [ ] **Step 1: Write `examples/service_account.rs`**

```rust
use onepassword_sdk::{Client, SecretsApi};

fn main() {
    let token = std::env::var("OP_SERVICE_ACCOUNT_TOKEN")
        .expect("Set OP_SERVICE_ACCOUNT_TOKEN environment variable");

    let client = Client::builder()
        .service_account_token(&token)
        .integration_info("My Rust App", "v1.0.0")
        .build()
        .expect("Failed to create client");

    let secret = client
        .secrets()
        .resolve("op://vault/item/field")
        .expect("Failed to resolve secret");

    println!("Secret: {secret}");
}
```

- [ ] **Step 2: Write `examples/desktop_app.rs`**

```rust
#[cfg(feature = "desktop")]
fn main() {
    use onepassword_sdk::{Client, SecretsApi};

    let client = Client::builder()
        .desktop_app_integration("my-account")
        .integration_info("My Rust App", "v1.0.0")
        .build()
        .expect("Failed to create client");

    let secret = client
        .secrets()
        .resolve("op://vault/item/field")
        .expect("Failed to resolve secret");

    println!("Secret: {secret}");
}

#[cfg(not(feature = "desktop"))]
fn main() {
    eprintln!("This example requires the 'desktop' feature: cargo run --example desktop_app --features desktop");
}
```

- [ ] **Step 3: Commit**

```bash
git add examples/
git commit -m "feat: add service_account and desktop_app examples"
```

---

### Task 12: CI Workflow

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Write `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: "1.93.1"
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Test
        run: cargo test

      - name: Test (desktop feature)
        run: cargo test --features desktop

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add fmt, clippy, test, and cargo-deny checks"
```

---

### Task 13: Final Verification

- [ ] **Step 1: Run fmt**

Run: `cargo fmt --check`
Expected: PASS

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: PASS (fix any warnings)

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 4: Run clippy with desktop feature**

Run: `cargo clippy --features desktop -- -D warnings`
Expected: PASS

- [ ] **Step 5: Verify the crate compiles with all features**

Run: `cargo build --all-features`
Expected: PASS

- [ ] **Step 6: Final commit (if any fixes were needed)**

```bash
git add -A
git commit -m "fix: address clippy and formatting issues"
```
