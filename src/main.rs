use std::collections::HashMap;
use log::{error, info};
use tonic::{
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

struct Launchpad {
    program: &'static str,
    discriminator: &'static [u8],
    init_idx: usize,
}

const GRPC_ENDPOINT: &str = "http://134.119.192.123:10000";

const RAYDIUM: Launchpad = Launchpad {
    program: "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
    discriminator: &[175, 175, 109, 31, 13, 152, 155, 237],
    init_idx: 6, // index of the 'initialize' instruction in the IDL
};
const PUMP_FUN: Launchpad = Launchpad {
    program: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
    discriminator: &[24, 30, 200, 40, 5, 28, 7, 119],
    init_idx: 6, // index of the 'create' instruction in the IDL
};

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
        "Raydium launchpad monitor".to_string(),
        SubscribeRequestFilterTransactions {
            account_include: vec![],
            account_exclude: vec![],
            account_required: vec![
                RAYDIUM.program.to_string(),
            ],
            vote: Some(false),
            failed: Some(false),
            signature: None,
        });
    accounts_filter.insert(
        "Pump.fun monitor".to_string(),
        SubscribeRequestFilterTransactions {
            account_include: vec![],
            account_exclude: vec![],
            account_required: vec![
                PUMP_FUN.program.to_string(),
            ],
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
                // let tx_id = bs58::encode(&tx_info.signature).into_string();
                if let Some(tx) = &tx_info.transaction {
                    if let Some(message) = &tx.message {
                        for instruction in &message.instructions {
                            let program_id = bs58::encode(&message.account_keys[instruction.program_id_index as usize]).into_string();

                            if program_id == PUMP_FUN.program && instruction.data.starts_with(&PUMP_FUN.discriminator) {
                                let base_mint_index = instruction.accounts[PUMP_FUN.init_idx] as usize;
                                let base_mint_pubkey = &message.account_keys[base_mint_index];
                                info!("Pump.fun token created! CA: {}", bs58::encode(base_mint_pubkey).into_string());
                            }

                            if program_id == RAYDIUM.program && instruction.data.starts_with(&RAYDIUM.discriminator) {
                                let base_mint_index = instruction.accounts[PUMP_FUN.init_idx] as usize;
                                let base_mint_pubkey = &message.account_keys[base_mint_index];
                                info!("Raydium LaunchLab token launched! CA: {}", bs58::encode(base_mint_pubkey).into_string());
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}