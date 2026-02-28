use anyhow::{Result, Context};
use aws_config::SdkConfig;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use aws_secretsmanager_caching::SecretsManagerCachingClient;
use aws_sdk_kms::Client as KmsClient;

use std::num::NonZeroUsize;
use std::time::Duration;


const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };
const CACHE_TTL: Duration = Duration::from_secs(300);

pub async fn create_secret_client(config: &SdkConfig) -> Result<SecretsManagerCachingClient> {
    let client = SecretsManagerClient::new(&config);

    let caching_client = SecretsManagerCachingClient::new(
        client,
        CACHE_SIZE,
        CACHE_TTL,
        false
    )
    .context("Failed to create secrets caching client")?;
    
    Ok(caching_client)
}

pub async fn fetch_secret(
    secret_client: &SecretsManagerCachingClient,
    secret_name: &str,
) -> Result<String> {
    println!("Fetching secret: {}", secret_name);

    let secret_value = secret_client
        .get_secret_value(secret_name, None, None, false)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to retrieve secret '{}': {}", secret_name, e))?;
    
    let secret_string = secret_value
        .secret_string
        .ok_or_else(|| anyhow::anyhow!("Secret '{}' has no string value", secret_name))?;
          
    Ok(secret_string)
}

pub async fn validate_credentials(kms_client: &KmsClient) -> Result<()> {
    kms_client
        .list_keys()
        .limit(1)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to validate AWS credentials: {}", e))?;
    
    log::debug!("AWS credentials validated successfully");
    Ok(())
}