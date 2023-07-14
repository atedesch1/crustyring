use std::net::SocketAddr;

use crustyring::{
    error::Result,
    registry::{service::RegistryService, REGISTRY_PORT},
    rpc::registry::registry_server::RegistryServer,
};
use log::info;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    
    let service = RegistryService::new();
    let addr: SocketAddr = format!("0.0.0.0:{}", REGISTRY_PORT).parse()?;
    
    let hostname = std::env::var("REGISTRY_HOSTNAME").unwrap_or("0.0.0.0".to_owned());
    let public_addr = format!("http://{}:{}", hostname, REGISTRY_PORT);
    info!("Initializing registry service on {}", public_addr);

    Server::builder()
        .add_service(RegistryServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
