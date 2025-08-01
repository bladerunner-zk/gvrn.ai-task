use crate::launchpad::{
    PUMP_FUN,
    RAYDIUM,
};
use std::collections::HashMap;
use futures::SinkExt;
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequest, SubscribeRequestFilterTransactions};

pub(crate) async fn send_subscription_request<T>(mut tx: T) -> Result<(), Box<dyn std::error::Error>>
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