
use nitro::socket::{Listener, Tcp};
use nitro::init_logging;

use std::net::Ipv4Addr;
use std::io;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::Client as SecretClient;
use aws_secretsmanager_caching::{
    SecretsManagerCachingClient, 
    SecretsManagerCachingClientBuilder
};

mod aws;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;

    let secret_client = aws::create_secret_client(&config)
    let kms_client = KmsClient::new(&config);

    let listener = VsockListener::bind(2, 3000)?;
    println!("Listening on vsock port 3000 (CID: 2)");

    // Accept connections in a loop
    loop {
        // BORROW CHECKER: accept() returns ownership of stream
        let (stream, peer_addr) = listener.accept().await?;
        println!("Connection from: {:?}", peer_addr);

        // BORROW CHECKER: Clone clients for each task
        // AWS SDK clients use Arc internally, so clone is cheap
        let secrets_client = secrets_client.clone();
        let kms_client = kms_client.clone();

        // BORROW CHECKER: Spawn task with 'move' to transfer ownership
        // The task needs to OWN these values to outlive this loop iteration
        tokio::spawn(async move {
            if let Err(e) = vsock::handle_client(stream, secrets_client, kms_client).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }    

    Ok(())
}

// fn main() -> io::Result<()> {
//     init_logging().expect("Failed to initialize logging");
    
//     log::info!("Starting TCP server on 0.0.0.0:1122");

//     let listener = Listener::<Tcp>::bind_tcp(
//         Ipv4Addr::new(0, 0, 0, 0),
//         1122
//     )?;
    
//     log::info!("Listening for connections...");
    
//     // Accept connections
//     loop {
//         let (stream, peer_addr) = listener.accept()?;
//         log::info!("New connection from: {}", peer_addr);
        
//         // Read data
//         let mut buffer = [0u8; 1024];
//         let n = stream.read(&mut buffer)?;
        
//         log::debug!("Received {} bytes", n);
        
//         // Echo back
//         stream.write(&buffer[..n])?;
//         log::info!("Echoed back {} bytes", n);
//     }
// }