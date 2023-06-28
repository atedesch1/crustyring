use std::{env, net::SocketAddr};

use crustyring::{
    dht::service::DhtNodeService,
    error::{Error, Result},
    rpc::dht::dht_node_server::DhtNodeServer,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = args
        .get(1)
        .ok_or(Error::Parse("missing port argument".into()))?;

    let addr: SocketAddr = format!("[::1]:{}", port).parse()?;
    let service = DhtNodeService::new(addr.to_string()).await?;

    println!("Running node on {}", addr);

    Server::builder()
        .add_service(DhtNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
