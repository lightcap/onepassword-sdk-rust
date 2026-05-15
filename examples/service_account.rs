use onepassword_sdk::{Client, SecretsApi};

fn main() {
    let token = std::env::var("OP_SERVICE_ACCOUNT_TOKEN")
        .expect("Set OP_SERVICE_ACCOUNT_TOKEN environment variable");

    let client = Client::builder()
        .service_account_token(&token)
        .integration_info("My Rust App", "v1.0.0")
        .build()
        .expect("Failed to create client");

    let secret_ref =
        std::env::var("OP_SECRET_REF").unwrap_or_else(|_| "op://vault/item/field".to_string());
    client
        .secrets()
        .resolve(&secret_ref)
        .expect("Failed to resolve secret");

    println!("Secret resolved successfully");
}
