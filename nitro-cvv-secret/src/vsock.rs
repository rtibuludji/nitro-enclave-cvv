use anyhow::{Result};

use tokio_vsock::VsockStream;
use tokio_util::sync::CancellationToken;

use nitro_tokio::message_utils::{read_message};



pub async fn handle_client(
    mut stream: VsockStream,
    shutdown_token: CancellationToken,
) -> Result<()> {
    log::info!("Client connected, started");
    
    loop {
        let message: Vec<u8> = match read_message(&mut stream, None, &shutdown_token).await {
            Ok(msg) => msg,
            Err(_)  => {
                log::error!("read message got error");
                break;
            }
        };

        if message.len() == 0 {
            log::warn!("recieve zero or timeout, force connection closed")
            // close the stream,
            // return err;
        }

    }

    log::info!("Connection closed");
    Ok(())
}