use anyhow::{Result, Context};
use tokio_vsock::{
    VsockAddr,
    VsockStream
};

pub async fn test(cid: u32, port: u32) -> Result<()> {
    
    let host_addr = VsockAddr::new(cid, port);
    println!("Connecting to vsock CID {} port {}/{}...", cid, port, host_addr);

    let mut _stream = VsockStream::connect(host_addr)
        .await
        .context(format!("Failed to connect to CID {} port {}", cid, port))?;
    
    println!("✓ Connected!");
    
    
    Ok(())
}