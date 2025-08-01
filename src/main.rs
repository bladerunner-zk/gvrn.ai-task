mod client;
mod constants;
mod launchpad;
mod subscription;
mod process_updates;
mod event;
mod mq;

use client::setup_client;
use subscription::send_subscription_request;
use process_updates::process_updates;
use constants::GRPC_ENDPOINT;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Connecting to gRPC enpoint at {}", GRPC_ENDPOINT);
    let mut client = setup_client().await?;
    info!("Connected to gRPC endpoint");

    let (subscribe_tx, subscribe_rx) = client.subscribe().await?;
    info!("Subscription stream established");

    send_subscription_request(subscribe_tx).await?;
    info!("Subscription request sent.");

    process_updates(subscribe_rx).await?;
    
    info!("Stream closed");
    Ok(())
}