use ethereum_types::H256;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use crate::types::Bytes;

pub use rpc_impl_Web3Api::gen_server::Web3Api as Web3ApiServer;

#[rpc(server)]
pub trait Web3Api {
    #[rpc(name = "web3_clientVersion")]
    fn client_version(&self) -> Result<String>;

    #[rpc(name = "web3_sha3")]
    fn sha3(&self, _: Bytes) -> Result<H256>;
}