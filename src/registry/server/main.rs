use crustyring::{
    error::Result,
    registry::{service::RegistryService, REGISTRY_ADDR},
    rpc::registry::registry_server::RegistryServer,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let service = RegistryService::new();

    Server::builder()
        .add_service(RegistryServer::new(service))
        .serve(REGISTRY_ADDR)
        .await?;

    Ok(())
}
