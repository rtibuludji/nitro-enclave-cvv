use anyhow::{Result};

use tokio_vsock::VsockStream;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use aws_sdk_kms::Client as KmsClient;

use nitro_tokio::message_utils::{read_message, write_message};

use nitro::utils;
use nitro::message::{
    Message,
    VerifyCVVRequest,
    VerifyCVVResponse,
};

use crate::aws;

pub async fn handle_client(
    mut stream: VsockStream,
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
            Message::VerifyCVVRequest(request) => {
                log::info!("Processing VerifyCVV request");

                let response = process_verifycvv(&request, &kms_client, &shutdown_token).await;

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

async fn process_verifycvv(
    request: &VerifyCVVRequest,
    _kms_client: &KmsClient,
    shutdown_token: &CancellationToken,
) -> VerifyCVVResponse {

    let hdr = request.header.hdr;

    let cvka_key_id = String::from_utf8_lossy(&request.cvka)
        .trim_end_matches('\0')
        .to_string();

    let cvkb_key_id = String::from_utf8_lossy(&request.cvkb)
        .trim_end_matches('\0')
        .to_string();

    log::info!("VerifyCVV: CVKA='{}', CVKB='{}'", cvka_key_id, cvkb_key_id);

    let encrypted_cvka = match aws::get_secret_key(&cvka_key_id, shutdown_token).await {
        Ok(key) => key,
        Err(e) => {
            log::error!("Failed to get CVKA: {}", e);
            return VerifyCVVResponse::error(hdr, *b"99");
        }
    };
    
    log::debug!("Got encrypted CVKA ({} bytes)", encrypted_cvka.len());

    return VerifyCVVResponse::error(hdr, *b"99");
}