use std::sync::Arc;
use tonic::{Request, Response, Status};

use super::manager::Manager;
use crate::rpc::registry::registry_server::Registry;
use crate::rpc::registry::{ConnectionAddr, Node, RegisterInfo};

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
    async fn register_node(
        &self,
        request: Request<ConnectionAddr>,
    ) -> std::result::Result<Response<RegisterInfo>, Status> {
        let conn_addr = &request.get_ref().addr;
        let id = self.manager.register_node(conn_addr.to_owned())?;
        println!("Registered addr: {} as #{}", conn_addr, id);

        let neighbor = self.manager.find_closest_neighbor(id)?.map(|node| Node {
            id: node.id,
            addr: node.addr,
        });

        Ok(Response::new(RegisterInfo { id, neighbor }))
    }
}
