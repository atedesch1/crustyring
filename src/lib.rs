pub mod error;
pub mod hash;

pub mod rpc;

pub mod dht;
pub mod registry;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    id: u64,
    addr: String,
}
