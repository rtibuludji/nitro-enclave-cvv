use anyhow::{Result};
use std::sync::Arc;

use tokio_vsock::VsockStream;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use nitro_tokio::message_utils::{read_message, write_message};

use nitro::utils;
use nitro::message::{
    Message,
    GetKeyResponse
};

use aws_secretsmanager_caching::SecretsManagerCachingClient;
use aws_sdk_kms::Client as KmsClient;


pub async fn handle_client(
    mut stream: VsockStream,
    _secret_client: Arc<SecretsManagerCachingClient>,
    _kms_client: KmsClient,
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
                
                // TODO: Process request and send response
                // let response = process_key_request(request).await?;
                // write(&mut stream, &response.to_bytes(), None, &shutdown_token, "response").await?;


                

                let response = GetKeyResponse::error(*b"0000", *b"01");
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