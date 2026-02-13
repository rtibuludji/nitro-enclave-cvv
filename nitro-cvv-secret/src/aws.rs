use anyhow::{Result, Context};
use aws_secretsmanager_caching::SecretsManagerCachingClient;
use serde_json::Value as JsonValue;
use std::error::Error;


pub async fn fetch_secret(
    secret_client: &SecretsManagerCachingClient,
    secret_name: &str,
) -> Result<String> {
    println!("Fetching secret: {}", secret_name);

    let secret_value = secret_client
        .get_secret_string(secret_name)
        .await
        .with_context(|| format!("Failed to retrieve secret '{}'", secret_name))?;
          
    Ok(secret_value)
}