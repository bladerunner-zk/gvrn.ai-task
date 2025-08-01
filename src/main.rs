use std::env;

use log::{error, info, warn};
use yellowstone_grpc_client::GeyserGrpcClient;
use tonic::service::Interceptor;

const GRPC_ENDPOINT: &str = "http://134.119.192.123:10000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Connecting to gRPC enpoint at {}", GRPC_ENDPOINT);
    let mut client = setup_client().await?;
    info!("Connected to gRPC endpoint");
    Ok(())
}

async fn setup_client() -> Result<GeyserGrpcClient<impl Interceptor>, Box<dyn std::error::Error>> {    
    let client = GeyserGrpcClient::build_from_shared(GRPC_ENDPOINT.to_string())?
        .connect()
        .await?;
    
    Ok(client)
}