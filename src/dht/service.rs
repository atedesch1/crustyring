use std::sync::Arc;

use crate::error::{Error, Result};
use crate::registry::REGISTRY_ADDR;
use crate::rpc::registry::{ConnectionAddr, Node};

use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::rpc::registry::registry_client::RegistryClient;

use crate::rpc::dht::dht_node_client::DhtNodeClient;
use crate::rpc::dht::dht_node_server::DhtNode;
use crate::rpc::dht::{
    NeighborRegisterInfo, NeighborType, OperationType, PreviousNeighbors, Query, QueryResult,
};

#[derive(Debug)]
pub struct Neighbor {
    id: u64,
    addr: String,
    client: DhtNodeClient<Channel>,
}

#[derive(Debug)]
pub struct NeighborConnections {
    prev_node: Mutex<Option<Neighbor>>,
    next_node: Mutex<Option<Neighbor>>,
}

#[derive(Debug)]
pub struct DhtNodeService {
    id: u64,
    addr: String,

    neighbors: Arc<NeighborConnections>,

    registry: RegistryClient<Channel>,
}

impl DhtNodeService {
    pub async fn new(addr: String) -> Result<Self> {
        let mut registry = Self::try_connect_registry().await?;
        let node_info = registry
            .register_node(Request::new(ConnectionAddr { addr: addr.clone() }))
            .await?;
        let node_info = node_info.get_ref().clone();

        let node = Node {
            id: node_info.id,
            addr: addr.clone(),
        };

        println!("Registered as #{}", node.id);

        let neighbors = Arc::new(NeighborConnections {
            prev: Mutex::new(None),
            next: Mutex::new(None),
        });

        if let Some(neighbor) = node_info.neighbor {
            tokio::spawn(Self::connect_to_neighbors(
                node.clone(),
                neighbors.clone(),
                neighbor,
            ));
        }

        Ok(DhtNodeService {
            id: node.id,
            addr: node.addr,
            neighbors,
            registry,
        })
    }

    async fn try_connect_registry() -> Result<RegistryClient<Channel>> {
        for attempt in 1..=5 {
            match RegistryClient::connect("http://".to_owned() + REGISTRY_ADDR).await {
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
            match DhtNodeClient::connect("http://".to_owned() + addr).await {
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

    pub async fn get_node_info(node: &Mutex<Option<Neighbor>>) -> Option<Node> {
        let node = node.lock().await;
        node.as_ref().map(|n| Node {
            id: n.id,
            addr: n.addr.clone(),
        })
    }

    pub async fn connect_to_neighbors(
        node: Node,
        neighbors: Arc<NeighborConnections>,
        prev_neighbor: Node,
    ) -> Result<()> {
        let previous_neighbors =
            Self::register_on_neighbor(&node, &neighbors, &prev_neighbor, NeighborType::Next)
                .await?;
        let next_neighbor = previous_neighbors.next.unwrap_or(prev_neighbor);
        let _ =
            Self::register_on_neighbor(&node, &neighbors, &next_neighbor, NeighborType::Previous)
                .await;
        Ok(())
    }

    pub async fn register_on_neighbor(
        node: &Node,
        neighbors: &Arc<NeighborConnections>,
        neighbor: &Node,
        ty: NeighborType,
    ) -> Result<PreviousNeighbors> {
        println!("Connecting to #{}", neighbor.id);
        let mut client = Self::try_connect_node(&neighbor.addr).await?;

        println!("Registering as next on #{}", neighbor.id);
        let previous_neighbors = client
            .register_as_neighbor(Request::new(NeighborRegisterInfo {
                ty: ty.into(),
                id: node.id,
                addr: node.addr.clone(),
            }))
            .await?;
        let mut guard = match ty {
            NeighborType::Previous => neighbors.prev.lock().await,
            NeighborType::Next => neighbors.next.lock().await,
        };
        *guard = Some(Neighbor {
            id: neighbor.id,
            addr: neighbor.addr.clone(),
            client,
        });

        Ok(previous_neighbors.get_ref().clone())
    }

    pub async fn switch_neighbor(&self, info: &NeighborRegisterInfo) -> Result<()> {
        let neighbor = match NeighborType::from_i32(info.ty).unwrap() {
            NeighborType::Previous => &self.neighbors.prev,
            NeighborType::Next => &self.neighbors.next,
        };

        let client = DhtNodeService::try_connect_node(&info.addr).await?;
        let mut neighbor = neighbor.lock().await;
        if let Some(node) = neighbor.as_ref() {
            match NeighborType::from_i32(info.ty).unwrap() {
                NeighborType::Previous => {
                    println!("Replacing prev #{} with #{}", node.id, info.id)
                }
                NeighborType::Next => println!("Replacing next #{} with #{}", node.id, info.id),
            }
        }

        *neighbor = Some(Neighbor {
            id: info.id,
            addr: info.addr.clone(),
            client,
        });

        Ok(())
    }
}

#[tonic::async_trait]
impl DhtNode for DhtNodeService {
    async fn register_as_neighbor(
        &self,
        request: Request<NeighborRegisterInfo>,
    ) -> std::result::Result<Response<PreviousNeighbors>, Status> {
        let register_info = request.into_inner();

        let previous_neighbors = PreviousNeighbors {
            prev: DhtNodeService::get_node_info(&self.neighbors.prev).await,
            next: DhtNodeService::get_node_info(&self.neighbors.next).await,
        };

        self.switch_neighbor(&register_info).await?;

        Ok(Response::new(previous_neighbors))
    }

    async fn query_dht(
        &self,
        request: Request<Query>,
    ) -> std::result::Result<Response<QueryResult>, Status> {
        todo!()
    }
}
