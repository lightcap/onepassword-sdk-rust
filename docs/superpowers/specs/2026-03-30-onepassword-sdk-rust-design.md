# 1Password Rust SDK — Design Spec

## Overview

Port of the [1Password Go SDK](https://github.com/1Password/onepassword-sdk-go/tree/v0.4.1-beta.1) to Rust. The SDK is a typed wrapper around an opaque `core.wasm` binary (and optionally a native shared library) that implements the 1Password protocol and cryptographic engine. The Rust SDK provides the same API surface as the Go SDK with idiomatic Rust patterns.

**Source of truth:** Go SDK at tag `v0.4.1-beta.1`.

**Conventions:** Follow patterns from the [cordon](https://github.com/codezero-llc/cordon) project — edition 2024, Rust 1.93.1, `thiserror`, `serde`, `cargo-deny`, inline tests, conventional commits.

## Architecture

```
┌─────────────────────────────────────────────┐
│  User Code                                  │
│  client.secrets().resolve("op://...")        │
├─────────────────────────────────────────────┤
│  Public API Layer                           │
│  SecretsApi, ItemsApi, VaultsApi, etc.      │
│  Typed Rust structs/enums ↔ JSON serde      │
├─────────────────────────────────────────────┤
│  Client                                     │
│  Builder pattern, invoke() dispatcher       │
├─────────────────────────────────────────────┤
│  Core Trait                                 │
│  init_client / invoke / release_client      │
├──────────────────┬──────────────────────────┤
│  ExtismCore      │  SharedLibCore           │
│  (WASM, default) │  (desktop feature, FFI)  │
│  include_bytes!  │  libloading + dlopen     │
│  Mutex<Plugin>   │  platform-specific       │
└──────────────────┴──────────────────────────┘
```

All API methods serialize parameters to JSON, call `Core::invoke()`, and deserialize the response. This matches the Go SDK's `clientInvoke()` pattern exactly.

## Crate Structure

Single crate, not a workspace. Matches the Go SDK's single-package approach.

```
onepassword-sdk-rust/
  Cargo.toml
  deny.toml
  rust-toolchain.toml
  .gitignore
  .github/
    workflows/
      ci.yml
  LICENSE
  README.md
  CLAUDE.md
  src/
    lib.rs                  # Crate root, public re-exports, #![deny(unsafe_code)]
    client.rs               # Client struct, ClientBuilder
    core.rs                 # Core trait definition
    core_extism.rs          # ExtismCore — WASM runtime (default)
    core_shared_lib.rs      # SharedLibCore — native FFI (desktop feature)
    core_shared_lib_unix.rs # dlopen path for macOS/Linux
    core_shared_lib_win.rs  # LoadLibrary path for Windows
    errors.rs               # SdkError enum (thiserror)
    types.rs                # All domain types (serde structs/enums)
    secrets.rs              # SecretsApi trait + implementation
    items.rs                # ItemsApi trait + implementation
    items_files.rs          # ItemsFilesApi trait + implementation
    items_shares.rs         # ItemsSharesApi trait + implementation
    vaults.rs               # VaultsApi trait + implementation
    groups.rs               # GroupsApi trait + implementation
    environments.rs         # EnvironmentsApi trait + implementation
  wasm/
    core.wasm               # Embedded WASM binary (~9.1MB)
  examples/
    service_account.rs
    desktop_app.rs
```

## Core Abstraction

```rust
pub(crate) trait Core: Send + Sync {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError>;
    fn release_client(&self, client_id: &[u8]);
}
```

### ExtismCore (default)

- Embeds `core.wasm` via `include_bytes!("../wasm/core.wasm")`
- Uses `extism` crate (Rust host SDK) to create and manage the plugin
- `Mutex<Plugin>` for thread-safe access (matches Go's `sync.Mutex` pattern)
- Implements host functions: `randomFill`, `getTime`, `getTimezoneOffset`
- 50MB message size limit (matches Go)

### SharedLibCore (feature: `desktop`)

- Loads `libop_sdk_ipc_client.{dylib,so,dll}` from the 1Password desktop app installation
- Uses `libloading` crate for cross-platform dynamic library loading
- `#[allow(unsafe_code)]` scoped to the FFI modules only
- Platform-specific discovery of the shared library path
- Separate modules for Unix (`dlopen`) and Windows (`LoadLibrary`) matching Go's `shared_lib_core_unix.go` / `shared_lib_core_windows.go` split

## Client Builder

```rust
use onepassword_sdk::{Client, ClientBuilder};

// Service account auth (default, WASM core)
let client = Client::builder()
    .service_account_token("ops_...")
    .integration_info("My App", "v1.0.0")
    .build()?;

// Desktop app auth (requires `desktop` feature)
let client = Client::builder()
    .desktop_app_integration("My App")
    .integration_info("My App", "v1.0.0")
    .build()?;
```

Builder validates that exactly one auth method is set (service account XOR desktop app), matching Go's mutual exclusivity check. `integration_info` defaults to `("Unknown", "Unknown")` if not set.

## Public API Surface

### SecretsApi

```rust
pub trait SecretsApi {
    fn resolve(&self, secret_reference: &str) -> Result<String, SdkError>;
    fn resolve_all(&self, secret_references: &[String]) -> Result<ResolveAllResponse, SdkError>;
}
```

Plus standalone functions (no client needed):
- `secrets::validate_secret_reference(reference: &str) -> Result<(), SdkError>`
- `secrets::generate_password(recipe: PasswordRecipe) -> Result<GeneratePasswordResponse, SdkError>`

### ItemsApi

```rust
pub trait ItemsApi {
    fn create(&self, params: ItemCreateParams) -> Result<Item, SdkError>;
    fn create_all(&self, vault_id: &str, params: &[ItemCreateParams]) -> Result<ItemsUpdateAllResponse, SdkError>;
    fn get(&self, vault_id: &str, item_id: &str) -> Result<Item, SdkError>;
    fn get_all(&self, vault_id: &str, item_ids: &[String]) -> Result<ItemsGetAllResponse, SdkError>;
    fn put(&self, item: Item) -> Result<Item, SdkError>;
    fn delete(&self, vault_id: &str, item_id: &str) -> Result<(), SdkError>;
    fn delete_all(&self, vault_id: &str, item_ids: &[String]) -> Result<ItemsDeleteAllResponse, SdkError>;
    fn archive(&self, vault_id: &str, item_id: &str) -> Result<(), SdkError>;
    fn list(&self, vault_id: &str, filters: &[ItemListFilter]) -> Result<Vec<ItemOverview>, SdkError>;
    fn shares(&self) -> &dyn ItemsSharesApi;
    fn files(&self) -> &dyn ItemsFilesApi;
}
```

### ItemsSharesApi

```rust
pub trait ItemsSharesApi {
    fn get_account_policy(&self, vault_id: &str, item_id: &str) -> Result<ItemShareAccountPolicy, SdkError>;
    fn validate_recipients(&self, policy: ItemShareAccountPolicy, recipients: &[String]) -> Result<Vec<ValidRecipient>, SdkError>;
    fn create(&self, item: Item, policy: ItemShareAccountPolicy, params: ItemShareParams) -> Result<String, SdkError>;
}
```

### ItemsFilesApi

```rust
pub trait ItemsFilesApi {
    fn attach(&self, item: Item, file_params: FileCreateParams) -> Result<Item, SdkError>;
    fn read(&self, vault_id: &str, item_id: &str, attr: FileAttributes) -> Result<Vec<u8>, SdkError>;
    fn delete(&self, item: Item, section_id: &str, field_id: &str) -> Result<Item, SdkError>;
    fn replace_document(&self, item: Item, doc_params: DocumentCreateParams) -> Result<Item, SdkError>;
}
```

### VaultsApi

```rust
pub trait VaultsApi {
    fn create(&self, params: VaultCreateParams) -> Result<Vault, SdkError>;
    fn list(&self, params: Option<VaultListParams>) -> Result<Vec<VaultOverview>, SdkError>;
    fn get_overview(&self, vault_id: &str) -> Result<VaultOverview, SdkError>;
    fn get(&self, vault_id: &str, params: VaultGetParams) -> Result<Vault, SdkError>;
    fn update(&self, vault_id: &str, params: VaultUpdateParams) -> Result<Vault, SdkError>;
    fn delete(&self, vault_id: &str) -> Result<(), SdkError>;
    fn grant_group_permissions(&self, vault_id: &str, group_permissions: &[GroupAccess]) -> Result<(), SdkError>;
    fn update_group_permissions(&self, group_permissions: &[GroupVaultAccess]) -> Result<(), SdkError>;
    fn revoke_group_permissions(&self, vault_id: &str, group_id: &str) -> Result<(), SdkError>;
}
```

### GroupsApi

```rust
pub trait GroupsApi {
    fn get(&self, group_id: &str, params: GroupGetParams) -> Result<Group, SdkError>;
}
```

### EnvironmentsApi

```rust
pub trait EnvironmentsApi {
    fn get_variables(&self, environment_id: &str) -> Result<GetVariablesResponse, SdkError>;
}
```

## Domain Types

All types ported from the Go SDK's `types.go` (which is itself generated from Rust via `typeshare`). Key mapping decisions:

| Go Pattern | Rust Pattern |
|---|---|
| Struct with exported fields | `pub struct` with `#[derive(Debug, Clone, Serialize, Deserialize)]` |
| Tagged union (`Type` + content field + manual JSON marshal) | `#[serde(tag = "type", content = "content")]` enum — native Rust, no manual impl |
| `[]string` | `Vec<String>` |
| `[]byte` | `Vec<u8>` |
| `*string` (optional) | `Option<String>` |
| Go constants (iota) | Rust enum variants |
| Bitmask permission constants | `pub const` values in a `permissions` module, or a bitflags type |

### Key type categories

- **Items:** `Item`, `ItemCreateParams`, `ItemOverview`, `ItemField`, `ItemSection`, `ItemCategory` (23 variants), `ItemFieldType` (15 variants), `ItemFieldDetails` (tagged enum: OTP/SSHKey/Address), `ItemState`
- **Files:** `FileAttributes`, `FileCreateParams`, `DocumentCreateParams`, `ItemFile`
- **Secrets:** `ResolveAllResponse`, `ResolvedReference`, `ResolveReferenceError` (17 variants), `PasswordRecipe` (tagged enum: Memorable/Pin/Random), `GeneratePasswordResponse`
- **Vaults:** `Vault`, `VaultOverview`, `VaultCreateParams`, `VaultUpdateParams`, `VaultGetParams`, `VaultListParams`, `VaultType` (5 variants)
- **Groups:** `Group`, `GroupGetParams`, `GroupType` (7 variants), `GroupState`, `GroupAccess`, `GroupVaultAccess`, `VaultAccess`
- **Sharing:** `ItemShareAccountPolicy`, `ItemShareParams`, `ItemShareDuration`, `ValidRecipient` (tagged enum), `AllowedType`, `AllowedRecipientType`
- **Environments:** `GetVariablesResponse`, `EnvironmentVariable`
- **Batch responses:** Generic `BatchResponse<T, E>` for `ItemsUpdateAllResponse`, `ItemsGetAllResponse`, `ItemsDeleteAllResponse`, `ResolveAllResponse`

## Error Handling

```rust
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

    #[error("desktop session expired")]
    DesktopSessionExpired,

    #[error("rate limit exceeded")]
    RateLimitExceeded,

    #[error("shared library error: {0}")]
    SharedLib(String),
}
```

Core errors arrive as JSON `{"name": "...", "message": "..."}` and are deserialized. `DesktopSessionExpired` triggers automatic client re-initialization and retry (matching Go behavior).

Batch operations use per-item error types rather than failing the whole batch, matching Go's `Response[T, E]` pattern.

## Resource Cleanup

`Client` implements `Drop` to call `core.release_client()`. This is cleaner than Go's `runtime.SetFinalizer` approach — deterministic cleanup rather than GC-dependent.

## Host Functions

The WASM core imports three host functions that we must provide:

| Function | Purpose | Implementation |
|---|---|---|
| `randomFill(ptr, len)` | Cryptographic random bytes | `getrandom` crate |
| `getTime()` | Current Unix timestamp (ms) | `std::time::SystemTime` |
| `getTimezoneOffset()` | UTC offset in minutes | `chrono::Local` or `time` crate |

These are registered as Extism host functions when creating the plugin.

## Feature Flags

| Feature | Default | Description |
|---|---|---|
| (none/default) | yes | WASM-based core, service account auth |
| `desktop` | no | Desktop app integration via native shared library |

## Dependencies

| Crate | Purpose |
|---|---|
| `extism` | WASM plugin host runtime |
| `serde`, `serde_json` | JSON serialization for core RPC |
| `thiserror` | Error derive macros |
| `getrandom` | Cryptographic random for WASM host function |
| `chrono` or `time` | Timezone offset for WASM host function |
| `libloading` | Dynamic library loading (desktop feature only) |

## Tooling & CI

From cordon conventions:

- `rust-toolchain.toml`: channel `1.93.1`, components `rustfmt`, `clippy`
- `deny.toml`: license allowlist, advisory db, ban wildcards, deny unknown sources
- CI workflow: `cargo fmt --check`, `cargo-deny check`, `cargo clippy -- -D warnings`, `cargo nextest run`, `cargo test --doc`
- `#![deny(unsafe_code)]` at crate root (not `forbid`, to allow scoped `#[allow]` on FFI modules)

## Not At Parity

These items cannot be achieved with the current approach and are flagged:

1. **Code generation pipeline** — The Go SDK types are generated by `op-codegen`/`typeshare` from the private Rust core repo. We're hand-porting them. If the core WASM updates, types must be manually reconciled. This is acceptable for an exploratory port.

2. **Automated core updates** — 1Password's `1PasswordSDKBot` pushes WASM updates to the Go/Python/JS SDKs automatically. Our repo won't receive these. Core updates require manually copying `core.wasm` from a reference SDK.

3. **Extism host function ABI** — The exact ABI for host functions (memory layout, calling convention) needs to be validated against the actual WASM binary. The Go SDK's implementation is the reference, but Rust's Extism host SDK may have slightly different ergonomics.
