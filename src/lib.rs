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

pub struct HashRing {}

impl HashRing {
    pub fn distance(a: u64, b: u64) -> u64 {
        let bigger: u64;
        let smaller: u64;

        if a > b {
            bigger = a;
            smaller = b;
        } else {
            bigger = b;
            smaller = a;
        }

        std::cmp::min(bigger - smaller, (u64::MAX - bigger) + smaller)
    }

    pub fn counter_clockwise_distance(a: u64, b: u64) -> u64 {
        if a > b {
            a - b
        } else {
            (u64::MAX - b) + a
        }
    }

    pub fn is_node_key(id: u64, next_id: u64, key: u64) -> bool {
        (id < next_id && (id <= key && key < next_id))
            || (id > next_id && (id <= key || key < next_id))
    }
}
