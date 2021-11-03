use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

pub use rpc_impl_NetApi::gen_server::NetApi as NetApiServer;

#[rpc(server)]
pub trait NetApi {
    #[rpc(name = "net_version")]
    fn version(&self) -> Result<String>;

    #[rpc(name = "net_peerCount")]
    fn peer_count(&self) -> Result<u32>;

    #[rpc(name = "net_listening")]
    fn is_listening(&self) -> Result<bool>;
}