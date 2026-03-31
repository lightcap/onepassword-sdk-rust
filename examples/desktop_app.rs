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
    eprintln!(
        "This example requires the 'desktop' feature: cargo run --example desktop_app --features desktop"
    );
}
