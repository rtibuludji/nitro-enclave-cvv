
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