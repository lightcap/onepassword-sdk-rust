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
- Do NOT add `Co-Authored-By: Claude` lines to commit messages

## Architecture

Typed wrapper around an opaque WASM core binary. All API calls serialize params to JSON,
call `Core::invoke()`, and deserialize the response. Two core backends:
- `ExtismCore` (default) — embedded WASM via Extism
- `SharedLibCore` (feature: `desktop`) — native shared library from 1Password desktop app
