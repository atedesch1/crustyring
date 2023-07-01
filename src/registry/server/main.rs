use std::net::SocketAddr;

use crustyring::{
    error::Result,
    registry::{service::RegistryService, REGISTRY_ADDR},
    rpc::registry::registry_server::RegistryServer,
};
use log::info;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let service = RegistryService::new();
    let addr: SocketAddr = REGISTRY_ADDR.to_owned().parse()?;

    info!("Initializing registry service on {}", addr);

    Server::builder()
        .add_service(RegistryServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
