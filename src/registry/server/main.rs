use std::net::SocketAddr;

use crustyring::{
    error::Result,
    registry::{service::RegistryService, REGISTRY_ADDR},
    rpc::registry::registry_server::RegistryServer,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let service = RegistryService::new();

    let addr: SocketAddr = REGISTRY_ADDR.to_owned().parse()?;
    println!("Running registry on {}", addr);

    Server::builder()
        .add_service(RegistryServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
