use std::sync::Arc;

use crate::error::{Error, Result};
use crate::hash::generate_hash64;
use crate::registry::service::RegistryService;
use crate::rpc::registry::{ConnectionAddr, Node};
use crate::HashRing;

use log::info;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::rpc::registry::registry_client::RegistryClient;

use crate::rpc::dht::dht_node_client::DhtNodeClient;
use crate::rpc::dht::dht_node_server::DhtNode;
use crate::rpc::dht::{
    EncodedQuery, KeyValueEntry, NeighborRegisterInfo, NeighborType, NodeId, OperationType,
    PreviousNeighbors, Query, QueryResult,
};

use super::store::Store;

#[derive(Debug)]
pub struct Neighbor {
    id: u64,
    addr: String,
    client: DhtNodeClient<Channel>,
}

#[derive(Debug)]
pub struct NeighborConnections {
    prev: RwLock<Option<Neighbor>>,
    next: RwLock<Option<Neighbor>>,
}

#[derive(Debug)]
pub struct DhtNodeService {
    id: u64,
    addr: String,

    store: Arc<Store>,
    neighbors: Arc<NeighborConnections>,

    registry: RegistryClient<Channel>,
}

impl DhtNodeService {
    pub async fn new(addr: String) -> Result<Self> {
        let mut registry = RegistryService::try_connect_registry().await?;

        info!("Registering on registry...");
        let node_info = registry
            .register_node(Request::new(ConnectionAddr { addr: addr.clone() }))
            .await?;
        let node_info = node_info.get_ref().clone();
        let node = Node {
            id: node_info.id,
            addr: addr.clone(),
        };
        info!("Registered as #{:x}", node.id);

        let neighbors = Arc::new(NeighborConnections {
            prev: RwLock::new(None),
            next: RwLock::new(None),
        });

        let store = Arc::new(Store::new());

        if let Some(neighbor) = node_info.neighbor {
            tokio::spawn(Self::setup_connections(
                node.clone(),
                store.clone(),
                neighbors.clone(),
                neighbor,
            ));
        }

        Ok(DhtNodeService {
            id: node.id,
            addr: node.addr,
            store,
            neighbors,
            registry,
        })
    }

    pub async fn try_connect_node(node: &Node) -> Result<DhtNodeClient<Channel>> {
        info!("Connecting to node #{:x}...", node.id);
        for attempt in 1..=5 {
            match DhtNodeClient::connect("http://".to_owned() + &node.addr).await {
                Ok(client) => {
                    info!("Connected to node #{:x}.", node.id);
                    return Ok(client);
                }
                Err(_) => {
                    println!(
                        "Connection to node attempt {} failed. Retrying in 5 seconds...",
                        attempt
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        Err(Error::Internal("Connection to node failed.".into()))
    }

    pub async fn get_node_info(node: &RwLock<Option<Neighbor>>) -> Option<Node> {
        let node = node.read().await;
        node.as_ref().map(|n| Node {
            id: n.id,
            addr: n.addr.clone(),
        })
    }

    pub async fn setup_connections(
        node: Node,
        store: Arc<Store>,
        neighbors: Arc<NeighborConnections>,
        prev_neighbor: Node,
    ) -> Result<()> {
        Self::connect_to_neighbors(&node, &neighbors, &prev_neighbor).await?;
        Self::get_keys_from_neighbor(&node, &neighbors, &store).await?;
        Ok(())
    }

    pub async fn connect_to_neighbors(
        node: &Node,
        neighbors: &Arc<NeighborConnections>,
        prev_neighbor: &Node,
    ) -> Result<()> {
        let previous_neighbors =
            Self::register_on_neighbor(&node, &neighbors, &prev_neighbor, NeighborType::Next)
                .await?;
        let next_neighbor = previous_neighbors.next.unwrap_or(prev_neighbor.clone());
        let _ =
            Self::register_on_neighbor(&node, &neighbors, &next_neighbor, NeighborType::Previous)
                .await?;
        Ok(())
    }

    pub async fn get_keys_from_neighbor(
        node: &Node,
        neighbors: &Arc<NeighborConnections>,
        store: &Arc<Store>,
    ) -> Result<()> {
        let prev_neighbor = neighbors.prev.read().await;
        if let Some(prev_neighbor) = (*prev_neighbor).as_ref() {
            let mut stream = prev_neighbor
                .client
                .clone()
                .transfer_keys(Request::new(NodeId { id: node.id }))
                .await?
                .into_inner();

            while let Some(kv_entry) = stream.message().await? {
                store.set(&kv_entry.key, &kv_entry.value).await;
            }
        }
        Ok(())
    }

    pub async fn register_on_neighbor(
        node: &Node,
        neighbors: &Arc<NeighborConnections>,
        neighbor: &Node,
        ty: NeighborType,
    ) -> Result<PreviousNeighbors> {
        let mut client = Self::try_connect_node(neighbor).await?;

        info!(
            "Registering as {} on #{:x}...",
            match ty {
                NeighborType::Next => "next",
                NeighborType::Previous => "previous",
            },
            neighbor.id
        );
        let previous_neighbors = client
            .register_as_neighbor(Request::new(NeighborRegisterInfo {
                ty: ty.into(),
                id: node.id,
                addr: node.addr.clone(),
            }))
            .await?;

        let mut guard = match ty {
            NeighborType::Next => neighbors.prev.write().await,
            NeighborType::Previous => neighbors.next.write().await,
        };
        *guard = Some(Neighbor {
            id: neighbor.id,
            addr: neighbor.addr.clone(),
            client,
        });

        info!("Registered on #{:x}", neighbor.id);

        Ok(previous_neighbors.get_ref().clone())
    }

    pub async fn switch_neighbor(&self, info: &NeighborRegisterInfo) -> Result<()> {
        let ty = NeighborType::from_i32(info.ty).ok_or(Error::Internal(format!(
            "Neighbor type {} is not valid",
            info.ty
        )))?;
        let neighbor = match ty {
            NeighborType::Previous => &self.neighbors.prev,
            NeighborType::Next => &self.neighbors.next,
        };

        let client = DhtNodeService::try_connect_node(&Node {
            id: info.id,
            addr: info.addr.clone(),
        })
        .await?;
        let mut neighbor = neighbor.write().await;
        *neighbor = Some(Neighbor {
            id: info.id,
            addr: info.addr.clone(),
            client,
        });

        info!(
            "Replaced {} neighbor with #{:x}",
            {
                match ty {
                    NeighborType::Previous => "previous",
                    NeighborType::Next => "next",
                }
            },
            info.id
        );

        Ok(())
    }

    pub async fn execute_query(&self, query: &EncodedQuery) -> Result<Option<Vec<u8>>> {
        let key = query.key;

        info!("Executing query for key {:x}.", query.key);

        match OperationType::from_i32(query.ty).unwrap() {
            OperationType::Set => {
                let value = query.value.clone();
                match value {
                    None => Err(Error::Value("Value not provided.".into())),
                    Some(value) => Ok(self.store.set(&key, &value).await),
                }
            }
            OperationType::Get => {
                let result = self.store.get(&key).await;
                match result {
                    None => Err(Error::Value("Key not present in database.".into())),
                    Some(_) => Ok(result),
                }
            }
            OperationType::Delete => {
                let result = self.store.delete(&key).await;
                match result {
                    None => Err(Error::Value("Key not present in database.".into())),
                    Some(_) => Ok(result),
                }
            }
        }
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
        let req = request.get_ref();

        self.forward_query(Request::new(EncodedQuery {
            ty: req.ty,
            key: generate_hash64(&req.key)?,
            value: req.value.clone(),
        }))
        .await
    }

    async fn forward_query(
        &self,
        request: Request<EncodedQuery>,
    ) -> std::result::Result<Response<QueryResult>, Status> {
        let req = request.get_ref();

        let key = req.key;

        info!("Received request for key {:x}", key);

        let next_neighbor = self.neighbors.next.read().await;

        let is_node_key = match next_neighbor.as_ref() {
            Some(next_neighbor) => HashRing::is_node_key(self.id, next_neighbor.id, key),
            None => true,
        };

        if is_node_key {
            let result = self.execute_query(&req).await;
            let query_result = QueryResult {
                value: result.clone().ok().flatten(),
                error: result.err().map(|e| e.to_string()),
            };

            return Ok(Response::new(query_result));
        }

        let next_neighbor = next_neighbor.as_ref().ok_or(Error::Internal(format!(
            "Missing next neighbor on node {:x}.",
            self.id
        )))?;

        let prev_neighbor = self.neighbors.prev.read().await;
        let prev_neighbor = prev_neighbor.as_ref().ok_or(Error::Internal(format!(
            "Missing previous neighbor on node {:x}.",
            self.id
        )))?;

        // If counter_clockwise_distance from node to key
        // < clockwise_distance from node to key
        let forwarding_neighbor = if HashRing::counter_clockwise_distance(self.id, key)
            < HashRing::counter_clockwise_distance(key, self.id)
        {
            prev_neighbor
        } else {
            next_neighbor
        };

        info!(
            "Forwarding request for key {:x} to #{:x}",
            key, forwarding_neighbor.id
        );
        forwarding_neighbor
            .client
            .clone()
            .forward_query(request)
            .await
    }

    type TransferKeysStream = ReceiverStream<std::result::Result<KeyValueEntry, Status>>;

    async fn transfer_keys(
        &self,
        request: Request<NodeId>,
    ) -> std::result::Result<Response<Self::TransferKeysStream>, Status> {
        let prev_id = self.id.clone();
        let id = request.get_ref().id;

        let (tx, rx) = mpsc::channel(100);

        let store = self.store.clone();
        tokio::spawn(async move {
            let is_node_key = |key: u64| !HashRing::is_node_key(prev_id, id, key);
            let entries = store.get_entries_satisfy(is_node_key).await;
            info!("Transferring keys to {:x}", request.get_ref().id);
            for (key, value) in entries {
                store.delete(&key).await;
                tx.send(Ok(KeyValueEntry { key, value })).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
