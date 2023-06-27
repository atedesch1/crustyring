use crustyring::{
    error::Result,
    registry::{rpc::registry_server::RegistryServer, service::RegistryService},
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50000".parse()?;
    let service = RegistryService::new();

    Server::builder()
        .add_service(RegistryServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
