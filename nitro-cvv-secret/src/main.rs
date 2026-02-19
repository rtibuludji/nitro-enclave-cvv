
use anyhow::{Result, Context};
use anyhow::anyhow;
use std::env;

use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_vsock::{
    VMADDR_CID_ANY,
    VsockAddr,
    VsockListener,
};
use nitro;

mod vsock;

const DEFAULT_PORT: u32 = 3000;
async fn run_server(listen_port: u32) -> Result<()> {

    let shutdown_token = CancellationToken::new();
    let server_token   = shutdown_token.clone();

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                println!("");
                log::info!("shutdown signal received ...");
                server_token.cancel();
            }
            Err(e) => {
                log::error!("failed to listen for shutdown signal: {}", e);
            }
        }    
    });    

    let addr     = VsockAddr::new(VMADDR_CID_ANY, listen_port);
    let listener = VsockListener::bind(addr).context(format!("failed to bind to cid: ANY port: {}", listen_port))?;

    log::info!("listening on cid: ANY port: {}", listen_port);
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((client_stream, client_addr)) => {
                        log::info!("accept connection from {}", client_addr);


                        let handler_token = shutdown_token.child_token();

                        tokio::spawn(async move {
                            if let Err(e) = vsock::handle_client(client_stream, handler_token).await {
                                log::error!("error handling client from {}: {}", client_addr, e);
                            }
                            else {
                                log::info!("client {} disconnected cleanly", client_addr);
                            }
                        });
                    },
                    Err(e) => {
                        log::error!("error accepting client connection: {}", e);
                    }
                }
            },
            _ = shutdown_token.cancelled() => {
                log::info!("shutdown initiated ...");
                break;            
            }
        }
    }

    log::info!("server stopped accepting new connections");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {

    nitro::init_logging().map_err(|e| anyhow!("Failed to initialize logging: {}", e))?;

    log::info!("starting ...");

    let listen_port = env::var("SECRET_PORT")
        .ok()
        .and_then(|v| {
            v.parse::<u32>().map_err(|e| {
                log::warn!(
                    "failed to parse SECRET_PORT='{}': {}, using default port {}",
                    v, e, DEFAULT_PORT
                );
                e
            }).ok()
        })
        .unwrap_or_else(|| {
            log::info!("SECRET_PORT not set, using default port {}", DEFAULT_PORT);
            DEFAULT_PORT
        });

    match run_server(listen_port).await {
        Ok(()) => {
            log::info!("server exited gracefully");
            Ok(())
        }
        Err(e) => {
            log::error!("server: {:?}", e);
            Err(e)
        }
    }
}

// use aws_config::meta::region::RegionProviderChain;
// use aws_config::BehaviorVersion;


// mod aws;


    // let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    // let config = aws_config::defaults(BehaviorVersion::latest())
    //     .region(region_provider)
    //     .load()
    //     .await;

    // let secret_client = aws::create_secret_client(&config).await?;

    // let kms_client = KmsClient::new(&config);

    // let addr     = VsockAddr::new(VMADDR_CID_ANY, 3000);
    // let listener = VsockListener::bind(addr)?;
    // println!("Listening on vsock port 3000 (CID: 2)");


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