use anyhow::{Result, Context};
use aws_config;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use aws_secretsmanager_caching::{
    SecretsManagerCachingClient,
    SecretsManagerCachingClientBuilder,
};

async fn create_secret_client(config: &aws_config) -> Result<SecretsManagerCachingClient> {
    let client = SecretsManagerClient::new(&config);

    let caching_client = SecretsManagerCachingClientBuilder::new()
        .client(client)
        .build()
        .await
        .context("Failed to build caching client")?;
    
    Ok(caching_client)
}

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