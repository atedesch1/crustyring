use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub mod manager;
pub mod service;

pub const REGISTRY_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 50000);
