use std::sync::Arc;

use crate::error::Result;

use tonic::{Request, Response, Status};

use super::manager::Manager;
use super::rpc::registry_server::Registry;
use super::rpc::{ConnectionAddr, Node, OperationType, Query, QueryResult, RegisterInfo};

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
}

#[tonic::async_trait]
impl Registry for RegistryService {
    async fn register_node(&self, request: Request<ConnectionAddr>) -> std::result::Result<Response<RegisterInfo>, Status> {
        let conn_addr = &request.get_ref().addr;
        let id = self.manager.register_node(conn_addr.to_owned())?;
        let neighbor = self.manager.find_closest_neighbor(id)?.map(|node| 
            Node {
                id: node.id,
                addr: node.addr,
            }
        );


        Ok(Response::new(RegisterInfo { id, neighbor }))
    }

    async fn query_dht(&self, request: Request<Query>) -> std::result::Result<Response<QueryResult>, Status> {
        todo!()
    }
}
