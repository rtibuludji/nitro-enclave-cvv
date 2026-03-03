use anyhow::{Result};
use std::sync::Arc;

use tokio_vsock::VsockStream;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use aws_secretsmanager_caching::SecretsManagerCachingClient;
use aws_sdk_kms::Client as KmsClient;

use nitro_tokio::message_utils::{read_message, write_message};

use nitro::utils;
use nitro::message::{
    Message,
    GetKeyRequest,
    GetKeyResponse,
};

use crate::aws;

pub async fn handle_client(
    mut stream: VsockStream,
    secret_client: Arc<SecretsManagerCachingClient>,
    kms_client: KmsClient,
    shutdown_token: CancellationToken,
) -> Result<()> {
    log::info!("Client connected, started");
    
    loop {
        let message_bytes: Vec<u8> = match read_message(&mut stream, None, &shutdown_token).await {
            Ok(bytes) => bytes,
            Err(e)  => {
                log::error!("read message error: {}", e);
                break;
            }
        };

        if message_bytes.is_empty() {
            log::info!("Connection closed by peer or timeout");
            break;
        }

        let dump = utils::hexdump_string(&message_bytes);
        log::info!("recieve message {} bytes\n\n{}", message_bytes.len(), dump);

        let message = match Message::parse(&message_bytes) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Failed to parse message: {}", e);
                continue;
            }
        };

        match message {
            Message::GetKeyRequest(request) => {
                log::info!("Key request for: {}", request.key_id_str());

                let response = process_key_request(&request, &secret_client, &kms_client).await;

                let timeout  = Some(Duration::from_secs(60));
                let written = write_message(
                    &mut stream,
                    &response.to_bytes(),
                    timeout,
                    &shutdown_token
                ).await?;
                
                if written == 0 {
                    log::error!("Failed to send request (connection closed or timeout)");
                }
                else {
                    let dump = utils::hexdump_string(&response.to_bytes());
                    log::info!("sent message {} bytes\n\n{}", written, dump);
                }
            }
            _ => {
                log::warn!("Server received unsupport request, ignoring");
            }
        }        
    }

    log::info!("Connection closed");
    Ok(())
}

async fn process_key_request(
    request: &GetKeyRequest,
    secrets_client: &SecretsManagerCachingClient,
    kms_client: &KmsClient,
) -> GetKeyResponse {

    let key_id = request.key_id_str();
    let hdr    = request.header.hdr;

    let kms_key_id = match std::env::var("KMS_KEY_ID") {
        Ok(id) if !id.is_empty() => id,
        _ => {
            log::warn!("KMS_KEY_ID not defined in environment use default nitro-kms-key");
            "alias/nitro-kms-key".to_string()
        }
    };

    let secret = match aws::fetch_secret(secrets_client, &key_id).await {
        Ok(s) => s,
        Err(e) => {
            let error_msg = e.to_string();
            log::error!("Failed to fetch secret '{}': {}", key_id, error_msg);

            if error_msg.contains("ResourceNotFoundException") 
                || error_msg.contains("not found") {
                log::warn!("Secret '{}' not found", key_id);
                return GetKeyResponse::error(hdr, *b"01");
            } else if error_msg.contains("AccessDenied") 
                || error_msg.contains("not authorized") {
                log::warn!("Access denied to secret '{}'", key_id);
                return GetKeyResponse::error(hdr, *b"97");
            } else {
                log::error!("Unknown error fetching secret: {}", error_msg);
                return GetKeyResponse::error(hdr, *b"99");
            }
        }
    };
    
    log::debug!("Secret fetched successfully ({} bytes)", secret.len());

    let encrypted = match aws::encrypt_with_kms(
        kms_client,
        &kms_key_id,
        secret.as_bytes()
    ).await {
        Ok(e) => e,
        Err(e) => {
            let error_msg = e.to_string();
            log::error!("Failed to encrypt with KMS: {}", error_msg);
            
            // Determine specific error code
            if error_msg.contains("AccessDenied") 
                || error_msg.contains("not authorized") {
                log::warn!("Access denied to KMS key '{}'", kms_key_id);
                return GetKeyResponse::error(hdr, *b"02");
            } else if error_msg.contains("NotFoundException")
                || error_msg.contains("not found") {
                log::warn!("KMS key '{}' not found", kms_key_id);
                return GetKeyResponse::error(hdr, *b"03");
            } else {
                log::error!("Encryption failed: {}", error_msg);
                return GetKeyResponse::error(hdr, *b"04");
            }
        }
    };
    
    log::info!("Successfully encrypted key '{}' ({} bytes)", key_id, encrypted.len());

    GetKeyResponse::success(hdr, encrypted)
}