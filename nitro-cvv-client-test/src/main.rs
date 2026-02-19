use anyhow::Result;

mod vsock_client;

#[tokio::main]
async fn main() -> Result<()> {
    vsock_client::test(1, 3000).await?;

    Ok(())
}
