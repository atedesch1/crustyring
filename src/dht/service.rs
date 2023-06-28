use std::sync::Arc;

use crate::error::{Error, Result};
use crate::registry::REGISTRY_ADDR;
use crate::rpc::registry::{ConnectionAddr, Node};

use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::rpc::registry::registry_client::RegistryClient;
use crate::rpc::registry::RegisterInfo;

use crate::rpc::dht::dht_node_client::DhtNodeClient;
use crate::rpc::dht::dht_node_server::DhtNode;
use crate::rpc::dht::{
    NeighborRegisterInfo, NeighborType, OperationType, PreviousNeighbors, Query, QueryResult,
};

#[derive(Debug)]
pub struct Neighbor {
    id: u64,
    client: DhtNodeClient<Channel>,
}

#[derive(Debug)]
pub struct DhtNodeService {
    id: u64,
    addr: String,

    prev_node: Option<Neighbor>,
    next_node: Option<Neighbor>,

    registry: RegistryClient<Channel>,
}

impl DhtNodeService {
    pub async fn new(addr: String) -> Result<Self> {
        let mut registry = Self::try_connect_registry().await?;
        let node_info = registry
            .register_node(Request::new(ConnectionAddr { addr: addr.clone() }))
            .await?;
        let id = node_info.get_ref().id;

        let mut prev_node = None;
        let mut next_node = None;

        if let Some(prev_neighbor) = &node_info.get_ref().neighbor {
            let mut client = Self::try_connect_node(&prev_neighbor.addr).await?;
            let previous_neighbors = client
                .register_as_neighbor(Request::new(NeighborRegisterInfo {
                    ty: NeighborType::Next.into(),
                    node: Some(Node {
                        id,
                        addr: addr.clone(),
                    }),
                }))
                .await?;
            prev_node = Some(Neighbor {
                id: prev_neighbor.id,
                client,
            });

            let next_neighbor = match &previous_neighbors.get_ref().next {
                Some(next_neighbor) => next_neighbor,
                None => prev_neighbor,
            };

            let mut client = Self::try_connect_node(&next_neighbor.addr).await?;
            client
                .register_as_neighbor(Request::new(NeighborRegisterInfo {
                    ty: NeighborType::Previous.into(),
                    node: Some(Node {
                        id,
                        addr: addr.clone(),
                    }),
                }))
                .await?;
            next_node = Some(Neighbor {
                id: next_neighbor.id,
                client,
            })
        }

        Ok(DhtNodeService {
            id,
            addr,
            prev_node,
            next_node,
            registry,
        })
    }

    async fn try_connect_registry() -> Result<RegistryClient<Channel>> {
        for attempt in 1..=5 {
            match RegistryClient::connect(REGISTRY_ADDR.to_string()).await {
                Ok(client) => {
                    return Ok(client);
                }
                Err(_) => {
                    println!("Registry client: connection to registry attempt {} failed. Retrying in 5 seconds...", attempt);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        Err(Error::Internal(
            "Registry client: connection to registry failed".into(),
        ))
    }

    async fn try_connect_node(addr: &str) -> Result<DhtNodeClient<Channel>> {
        for attempt in 1..=5 {
            match DhtNodeClient::connect(addr.to_string()).await {
                Ok(client) => {
                    return Ok(client);
                }
                Err(_) => {
                    println!("Node client: connection to node attempt {} failed. Retrying in 5 seconds...", attempt);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        Err(Error::Internal(
            "Node client: connection to node failed".into(),
        ))
    }
}

#[tonic::async_trait]
impl DhtNode for DhtNodeService {
    async fn register_as_neighbor(
        &self,
        request: Request<NeighborRegisterInfo>,
    ) -> std::result::Result<Response<PreviousNeighbors>, Status> {
        todo!()
    }

    async fn query_dht(
        &self,
        request: Request<Query>,
    ) -> std::result::Result<Response<QueryResult>, Status> {
        todo!()
    }
}
