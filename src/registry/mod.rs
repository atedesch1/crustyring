pub mod manager;
pub mod rpc {
    tonic::include_proto!("registry");
}
pub mod service;
