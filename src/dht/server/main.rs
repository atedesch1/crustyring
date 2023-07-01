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

    let addr: SocketAddr = format!("[::1]:{}", port).parse()?;
    let service = DhtNodeService::new(addr.to_string()).await?;
    info!("Initializing node on {}", addr);

    Server::builder()
        .add_service(DhtNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
