use crate::{
    launchpad::{
        PUMP_FUN,
        RAYDIUM,
    },
    mq::send_to_mq,
    event::LaunchEvent,
};
use futures::StreamExt;
use yellowstone_grpc_proto::{
    geyser::SubscribeUpdate,
    prelude::{subscribe_update::UpdateOneof, CompiledInstruction, Message},
};
use tonic::Status;
use log::{error, info};

pub(crate) async fn process_updates<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>> 
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
                if let Some(tx) = &tx_info.transaction {
                    if let Some(message) = &tx.message {
                        for instruction in &message.instructions {
                            let program_id = bs58::encode(&message.account_keys[instruction.program_id_index as usize]).into_string();
                            detect_launch(&program_id, instruction, message, &tx_id);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn detect_launch(program_id: &str, instruction: &CompiledInstruction, message: &Message, tx_id: &String) {
    for launchpad in &[PUMP_FUN, RAYDIUM] {
        if program_id == launchpad.program && instruction.data.starts_with(launchpad.discriminator) {
            let base_mint_index = instruction.accounts[launchpad.init_idx] as usize;
            let base_mint_pubkey = &message.account_keys[base_mint_index];
            let ca = bs58::encode(base_mint_pubkey).into_string();
            let event = LaunchEvent {
                launchpad: launchpad.name,
                ca,
                transaction_id: tx_id.clone(),
            };
            info!("{} token created; CA: {}; transaction signature: {}", event.launchpad, event.ca, tx_id);

            let _ = send_to_mq(&event);
        }
    }
}