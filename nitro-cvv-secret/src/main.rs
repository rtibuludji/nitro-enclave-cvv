
use anyhow::{Result, Context};
use anyhow::anyhow;
use std::env;
use std::sync::Arc;

use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_vsock::{
    VMADDR_CID_ANY,
    VsockAddr,
    VsockListener,
};

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_kms::Client as KmsClient;
use aws_secretsmanager_caching::SecretsManagerCachingClient;

use nitro;

mod aws;
mod session;

const DEFAULT_PORT: u32 = 3000;
async fn run_server(listen_port: u32, aws_secret_client: Arc<SecretsManagerCachingClient>, aws_kms_client: KmsClient) -> Result<()> {

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

                        let secret_client  = Arc::clone(&aws_secret_client);
                        let kms_client     = aws_kms_client.clone();
                        let handler_token  = shutdown_token.child_token();

                        tokio::spawn(async move {
                            if let Err(e) = session::handle_client(
                                client_stream, 
                                secret_client,
                                kms_client,
                                handler_token).await {
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

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let secret_client = Arc::new(aws::create_secret_client(&config).await?);
    let kms_client    = KmsClient::new(&config);
    log::info!("AWS clients initialized");

    log::info!("Validating AWS credentials...");
    match aws::validate_credentials(&config).await {
        Ok(()) => log::info!("AWS credentials validated successfully"),
        Err(e) => {
            log::error!("AWS credential validation failed: {}", e);
            return Err(e);
        }
    }

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

    match run_server(listen_port, secret_client, kms_client).await {
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
