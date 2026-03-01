use anyhow::{Result, Context};
use aws_config::SdkConfig;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use aws_sdk_kms::Client as KmsClient;
use aws_sdk_sts::Client as StsClient;

use aws_secretsmanager_caching::SecretsManagerCachingClient;

use std::num::NonZeroUsize;
use std::time::Duration;


const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1000) };
const CACHE_TTL: Duration = Duration::from_secs(300);

pub async fn validate_credentials(config: &SdkConfig) -> Result<()> {
    log::info!("Validating AWS credentials...");
    
    let sts_client = StsClient::new(config);
    
    match sts_client.get_caller_identity().send().await {
        Ok(identity) => {
            log::info!("✓ AWS credentials validated successfully");
            if let Some(account) = identity.account() {
                log::info!("  Account: {}", account);
            }
            if let Some(arn) = identity.arn() {
                log::info!("  ARN: {}", arn);
            }
            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!("Failed to validate AWS credentials: {}", e))
        }
    }
}

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

pub async fn encrypt_with_kms(
    kms_client: &KmsClient,
    key_id: &str,
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    log::debug!("Encrypting {} bytes with KMS key: {}", plaintext.len(), key_id);
 
    let blob = aws_sdk_kms::primitives::Blob::new(plaintext.to_vec());
 
    let response = kms_client
        .encrypt()
        .key_id(key_id)
        .plaintext(blob)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("KMS Encrypt API error: {}", e))?;

    let ciphertext_blob = response
        .ciphertext_blob()
        .ok_or_else(|| anyhow::anyhow!("KMS response contains no ciphertext"))?;

    let encrypted_bytes = ciphertext_blob.clone().into_inner();
    
    log::debug!(
        "Successfully encrypted with KMS (plaintext: {} bytes → ciphertext: {} bytes)",
        plaintext.len(),
        encrypted_bytes.len()
    );
    
    Ok(encrypted_bytes)
}
