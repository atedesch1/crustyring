pub mod hash;
pub mod error;
pub mod registry;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    id: u64,
    addr: String,
}
