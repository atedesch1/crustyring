use std::sync::Mutex;

use crate::{error::Result, hash, NodeInfo};

#[derive(Debug)]
pub struct Manager {
    nodes: Mutex<Vec<NodeInfo>>,
}

impl Manager {
    pub fn new() -> Self {
        Manager { nodes: Mutex::new(Vec::new()) }
    }

    pub fn register_node(&self, addr: String) -> Result<u64> {
        let hashed_id = hash::generate_id_hash(&addr)?;
        let mut nodes = self.nodes.lock()?;
        nodes.push(NodeInfo {
            id: hashed_id,
            addr,
        });

        Ok(hashed_id)
    }

    pub fn find_closest_neighbor(&self, id: u64) -> Result<Option<NodeInfo>> {
        let mut smallest_distance = std::u64::MAX;
        let mut result: Option<NodeInfo> = None;

        let nodes = self.nodes.lock()?;

        for node in nodes.as_slice() {
            let distance: u64 = if node.id > id {
                u64::MAX - (node.id - id)
            } else {
                id - node.id
            };

            if distance < smallest_distance {
                smallest_distance = distance;
                result = Some(node.clone());
            }
        }

        Ok(result)
    }
}
