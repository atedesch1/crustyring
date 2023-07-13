use crate::error::{Error, Result};
use crate::registry::REGISTRY_ADDR;
use crate::rpc::registry::registry_client::RegistryClient;
use log::{info, warn};
use std::sync::Arc;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use super::manager::Manager;
use crate::rpc::registry::registry_server::Registry;
use crate::rpc::registry::{ConnectionAddr, Node, Nodes, RegisterInfo};

#[derive(Debug)]
pub struct RegistryService {
    manager: Arc<Manager>,
}

impl RegistryService {
    pub fn new() -> Self {
        RegistryService {
            manager: Arc::new(Manager::new()),
        }
    }

    pub async fn try_connect_registry() -> Result<RegistryClient<Channel>> {
        info!("Connecting to registry...");
        for attempt in 1..=5 {
            match RegistryClient::connect("http://".to_owned() + REGISTRY_ADDR).await {
                Ok(client) => {
                    info!("Connected to registry.");
                    return Ok(client);
                }
                Err(_) => {
                    warn!(
                        "Connection to registry attempt {} failed. Retrying in 5 seconds...",
                        attempt
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        Err(Error::Internal("Connection to registry failed.".into()))
    }
}

#[tonic::async_trait]
impl Registry for RegistryService {
    async fn register_node(
        &self,
        request: Request<ConnectionAddr>,
    ) -> std::result::Result<Response<RegisterInfo>, Status> {
        let conn_addr = &request.get_ref().addr;
        info!("Received join request from address {}", conn_addr);

        info!("Registering node on address {}", conn_addr);
        let id = self.manager.register_node(conn_addr.to_owned())?;
        info!("Registered {} as #{:x}", conn_addr, id);

        let neighbor = self.manager.find_closest_neighbor(id)?.map(|node| Node {
            id: node.id,
            addr: node.addr,
        });

        Ok(Response::new(RegisterInfo { id, neighbor }))
    }

    async fn get_connected_nodes(
        &self,
        _request: Request<()>,
    ) -> std::result::Result<Response<Nodes>, Status> {
        let nodes = self
            .manager
            .get_nodes()?
            .iter()
            .map(|node| Node {
                id: node.id,
                addr: node.addr.clone(),
            })
            .collect();

        Ok(Response::new(Nodes { nodes }))
    }
}
