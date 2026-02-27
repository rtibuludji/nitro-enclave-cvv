use anyhow::{Result, Context};
use tokio_vsock::{
    VsockAddr,
    VsockStream
};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

use nitro::hexdump;
use nitro::message::{GetKeyRequest};
use nitro_tokio::message_utils::{write_message, read_message};

pub async fn test(cid: u32, port: u32) -> Result<()> {

    let key_id: &str = "test.test.cvka";

    let host_addr = VsockAddr::new(cid, port);
    println!("Connecting to vsock CID {} port {}/{}...", cid, port, host_addr);

    let mut stream = VsockStream::connect(host_addr)
        .await
        .context(format!("Failed to connect to CID {} port {}", cid, port))?;

    println!("✓ Connected!");

    let request = GetKeyRequest::new(*b"0000", key_id.as_bytes().to_vec());    
    println!("→ Sending Key Request:");
    hexdump(&request.to_bytes());

    let shutdown_token = CancellationToken::new();
    let timeout = Some(Duration::from_secs(60));

    let written = write_message(
        &mut stream,
        &request.to_bytes(),
        timeout,
        &shutdown_token
    ).await?;
    
    if written == 0 {
        anyhow::bail!("Failed to send request (connection closed or timeout)");
    }
    
    println!("✓ Sent {} bytes", written);

    let message_bytes: Vec<u8> = match read_message(&mut stream, timeout, &shutdown_token).await {
        Ok(bytes) => bytes,
        Err(e)  => {
            println!("read message error: {}", e);
            Vec::<u8>::new()
        }
    };
    println!("→ Recv Key Response:");
    hexdump(&message_bytes);

    Ok(())
}