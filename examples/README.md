# 1Password Rust SDK Examples

## Prerequisites

- A [1Password Service Account](https://developer.1password.com/docs/service-accounts/) token, or the 1Password desktop app installed and running
- A vault ID to use for item operations (the service account must have full CRUD permissions on this vault)
- `ssh-keygen` available on PATH (used to generate SSH keys for the SSH key example)

## Examples

### `service_account.rs` — Minimal service account auth

Authenticates with a service account token and resolves a single secret.

```sh
export OP_SERVICE_ACCOUNT_TOKEN="ops_..."
export OP_SECRET_REF="op://vault/item/field"  # optional, defaults to op://vault/item/field
cargo run --example service_account
```

### `example.rs` — Comprehensive API usage

Demonstrates the full SDK surface using a service account:

- **Secrets**: resolve single and batch references, validate references
- **Passwords**: generate PIN, memorable, and random passwords
- **Items**: create with fields/sections/tags/websites, list with filters, update, archive, delete
- **Batch operations**: create, get, and delete multiple items at once
- **Item sharing**: get account policy, validate recipients, create share link
- **SSH keys**: create an SSH key item (generates an Ed25519 key at runtime)
- **Documents**: create a document item, replace its content, read it back
- **File fields**: attach a file to an item, read it, delete the file field
- **Vaults**: create, list, get overview, get with accessors, update, delete

```sh
export OP_SERVICE_ACCOUNT_TOKEN="ops_..."
export OP_VAULT_ID="..."
export OP_SHARE_RECIPIENT="user@example.com"  # optional, for item sharing demo
cargo run --example example
```

### `desktop_app.rs` — Desktop app integration

Demonstrates the same operations using the 1Password desktop app for authentication, plus group permission management (grant, update, revoke).

```sh
export OP_ACCOUNT="my.1password.com"
export OP_VAULT_ID="..."
export OP_GROUP_ID="..."  # optional, for group permissions demo
cargo run --example desktop_app --features desktop
```

## Operations Covered

| Operation | `example.rs` | `desktop_app.rs` |
|---|:-:|:-:|
| Authenticate (service account) | ✓ | |
| Authenticate (desktop app) | | ✓ |
| Resolve secrets | ✓ | ✓ |
| Resolve all (batch) | ✓ | |
| Validate secret reference | ✓ | |
| Generate passwords | ✓ | |
| Create item (fields, sections, tags, websites) | ✓ | |
| List items (with filters) | ✓ | ✓ |
| Update item | ✓ | |
| Archive item | ✓ | |
| Delete item | ✓ | |
| Batch create/get/delete | ✓ | ✓ |
| Share item | ✓ | |
| SSH key item | ✓ | |
| Document item (create, replace, read) | ✓ | |
| File field (attach, read, delete) | ✓ | |
| List vaults | ✓ | ✓ |
| Vault CRUD | ✓ | ✓ |
| Group permissions (grant, update, revoke) | | ✓ |
