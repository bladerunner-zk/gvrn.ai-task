use crate::constants::GRPC_ENDPOINT;
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};

pub(crate) async fn setup_client() -> Result<GeyserGrpcClient<impl Interceptor>, Box<dyn std::error::Error>> {    
    let client = GeyserGrpcClient::build_from_shared(GRPC_ENDPOINT.to_string())?
        .connect()
        .await?;
    
    Ok(client)
}