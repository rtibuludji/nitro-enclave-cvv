
use anyhow::{Result, anyhow, Context};

use tokio::time::Duration;
use tokio_vsock::{VsockAddr, VsockStream, VMADDR_CID_HOST};
use tokio_util::sync::CancellationToken;

use aws_nitro_enclaves_nsm_api::api::{Request, Response};
use aws_nitro_enclaves_nsm_api::driver as nsm_driver;

use aws_sdk_kms::Client as KmsClient;
use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::types::RecipientInfo;

use serde_bytes::ByteBuf;

use nitro::message::{Message, GetKeyRequest};
use nitro_tokio::message_utils::{read_message, write_message};

pub fn get_attestation_document(
    user_data: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
) -> Result<Vec<u8>> {

    let nsm_fd = nsm_driver::nsm_init();
    if nsm_fd < 0 {
        return Err(anyhow!("Failed to initialize NSM (fd: {})", nsm_fd));
    }

    let request = Request::Attestation {
        user_data: user_data.map(ByteBuf::from),
        nonce: nonce.map(ByteBuf::from),
        public_key: public_key.map(ByteBuf::from),
    };

    let response = nsm_driver::nsm_process_request(nsm_fd, request);

    nsm_driver::nsm_exit(nsm_fd);

    match response {
        Response::Attestation { document } => {
            log::debug!("Attestation document received ({} bytes)", document.len());
            Ok(document)
        }
        Response::Error(err) => {
            Err(anyhow!("NSM returned error: {:?}", err))
        }
        _ => {
            Err(anyhow!("Unexpected NSM response type"))
        }
    }
}

pub async fn decrypt_with_attestation(
    kms_client: &KmsClient,
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    log::debug!("Decrypting with KMS attestation ({} bytes)", ciphertext.len());

    let attestation_doc = get_attestation_document(None, None, None)
        .context("Failed to get attestation from NSM")?;
    
    log::debug!("Attestation: {} bytes", attestation_doc.len());

    let recipient = RecipientInfo::builder()
        .attestation_document(Blob::new(attestation_doc))
        .key_encryption_algorithm(
            aws_sdk_kms::types::KeyEncryptionMechanism::RsaesOaepSha256
        )
        .build();

    log::debug!("Calling KMS Decrypt API (via vsock-proxy)");
    
    let response = kms_client
        .decrypt()
        .ciphertext_blob(Blob::new(ciphertext.to_vec()))
        .recipient(recipient)
        .send()
        .await
        .context("KMS Decrypt API failed")?;

    let plaintext = response
        .plaintext()
        .ok_or_else(|| anyhow!("KMS response contains no plaintext"))?;
    
    log::debug!("✓ Decrypted successfully ({} bytes)", plaintext.as_ref().len());
    
    Ok(plaintext.as_ref().to_vec())
}

pub async fn get_secret_key(
    key_id: &str,
    shutdown_token: &CancellationToken,
) -> Result<Vec<u8>> {

    let secret_port = 3000;
    let addr = VsockAddr::new(VMADDR_CID_HOST, secret_port);
    
    log::debug!("Connecting to secret server at CID {} port {}...", VMADDR_CID_HOST, secret_port);
    
    let mut stream = VsockStream::connect(addr)
        .await
        .context(format!("Failed to connect to secret server on port {}", secret_port))?;
    
    log::debug!("Connected to secret server");

    let request = match GetKeyRequest::new(*b"0000", key_id.as_bytes().to_vec()) {
        Ok(req) => req,
        Err(e)  => {
            log::error!("GetKeyRequest error: {}", e);
            return Err(e);
        }
    };

    let timeout = Some(Duration::from_secs(60));
    
    log::debug!("Requesting key: '{}'", key_id);

    let written = write_message(
        &mut stream,
        &request.to_bytes(),
        timeout,
        shutdown_token
    ).await?;
    
    if written == 0 {
        return Err(anyhow!("Failed to send request to secret server"));
    }
    
    log::debug!("Request sent ({} bytes)", written);

    let response_bytes = read_message(&mut stream, timeout, shutdown_token)
        .await?;
    
    if response_bytes.is_empty() {
        return Err(anyhow!("No response from secret server"));
    }
    
    log::debug!("Response received ({} bytes)", response_bytes.len());

    let message = Message::parse(&response_bytes)
        .context("Failed to parse secret server response")?;
    
    match message {
        Message::GetKeyResponse(response) => {
            if !response.is_success() {
                return Err(anyhow!(
                    "Secret server returned error: {}",
                    response.response_code_str()
                ));
            }
            
            let encrypted_key = response.encrypted_key
                .ok_or_else(|| anyhow!("Response indicated success but no encrypted key"))?;
            
            log::debug!("✓ Got encrypted key from secret server ({} bytes)", encrypted_key.len());
            
            Ok(encrypted_key)
        }
        _ => Err(anyhow!("Unexpected message type from secret server")),
    }
}

