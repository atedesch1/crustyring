use std::{env, net::SocketAddr};

use crustyring::{
    dht::service::DhtNodeService,
    error::{Error, Result},
    rpc::dht::dht_node_server::DhtNodeServer,
};
use log::info;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = args
        .get(1)
        .ok_or(Error::Parse("missing port argument".into()))?;

    env_logger::init();

    let hostname = std::env::var("NODE_HOSTNAME").unwrap_or("0.0.0.0".to_owned());
    let public_addr = format!("http://{}:{}", hostname, port);

    info!("Initializing node on {}", public_addr);
    let service = DhtNodeService::new(public_addr).await?;
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    Server::builder()
        .add_service(DhtNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
