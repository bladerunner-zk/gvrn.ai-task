use std::{collections::HashMap, env};
use log::{error, info, warn};
use tonic::{
    transport::ClientTlsConfig,
    service::Interceptor,
    Status,
};
use futures::{sink::SinkExt, stream::StreamExt};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::{
    geyser::SubscribeUpdate,
    prelude::{
        CommitmentLevel,
        SubscribeRequest,
        SubscribeRequestFilterTransactions,
        subscribe_update::UpdateOneof,
    },
};

const GRPC_ENDPOINT: &str = "http://134.119.192.123:10000";

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

async fn setup_client() -> Result<GeyserGrpcClient<impl Interceptor>, Box<dyn std::error::Error>> {    
    let client = GeyserGrpcClient::build_from_shared(GRPC_ENDPOINT.to_string())?
        .connect()
        .await?;
    
    Ok(client)
}

async fn send_subscription_request<T>(mut tx: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: SinkExt<SubscribeRequest> + Unpin,
    <T as futures::Sink<SubscribeRequest>>::Error: std::error::Error + 'static,
{
    let mut accounts_filter = HashMap::new();
    accounts_filter.insert(
        "account_monitor".to_string(),
        SubscribeRequestFilterTransactions {
            account_include: vec![],
            account_exclude: vec![],
            account_required: vec![],
            vote: Some(false),
            failed: Some(false),
            signature: None,
        },
    );
    tx.send(SubscribeRequest {
        transactions: accounts_filter,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    })
    .await?;

    Ok(())
}

async fn process_updates<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>> 
where 
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => handle_message(msg),
            Err(e) => {
                error!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn handle_message(msg: SubscribeUpdate) {
    match msg.update_oneof {
        Some(UpdateOneof::Transaction(tx_update)) => {
            if let Some(tx_info) = tx_update.transaction {
                let tx_id = bs58::encode(&tx_info.signature).into_string();
                info!("Transaction signature: {}", tx_id);
            }
        }
        _ => {}
    }
}