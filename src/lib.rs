#![deny(unsafe_code)]

mod client;
mod core;
#[cfg(feature = "wasm")]
mod core_extism;
mod environments;
mod errors;
mod groups;
mod items;
mod items_files;
mod items_shares;
mod secrets;
mod types;
mod vaults;

#[cfg(feature = "desktop")]
#[allow(unsafe_code)]
mod core_shared_lib;

pub use client::{Client, ClientBuilder};
pub use environments::EnvironmentsApi;
pub use errors::SdkError;
pub use groups::GroupsApi;
pub use items::ItemsApi;
pub use items_files::ItemsFilesApi;
pub use items_shares::ItemsSharesApi;
pub use secrets::{Secrets, SecretsApi};
pub use types::*;
pub use vaults::VaultsApi;
