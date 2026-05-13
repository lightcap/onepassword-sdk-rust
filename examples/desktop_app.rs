#[cfg(feature = "desktop")]
fn main() {
    use onepassword_sdk::{Client, SecretsApi};

    let account = std::env::var("OP_ACCOUNT")
        .expect("Set OP_ACCOUNT to your 1Password sign-in address (e.g. my.1password.com)");
    let secret_ref = std::env::var("OP_SECRET_REF")
        .unwrap_or_else(|_| "op://MyVault/Example/username".to_string());

    let client = Client::builder()
        .desktop_app_integration(&account)
        .integration_info("1Password Rust SDK Example", "v0.1.0")
        .build()
        .expect("Failed to create client");

    let secret = client
        .secrets()
        .resolve(&secret_ref)
        .expect("Failed to resolve secret");

    println!("Secret length: {}", secret.len());
}

#[cfg(not(feature = "desktop"))]
fn main() {
    eprintln!(
        "This example requires the 'desktop' feature: cargo run --example desktop_app --features desktop"
    );
}
