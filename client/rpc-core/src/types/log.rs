use crate::types::Bytes;
use ethereum_types::{H160, H256, U256};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub address: H160,
    pub topics: Vec<H256>,
    pub data: Bytes,
    pub block_hash: Option<H256>,
    pub block_number: Option<U256>,
    pub transaction_hash: Option<H256>,
    pub transaction_index: Option<U256>,
    pub log_index: Option<U256>,
    pub transaction_log_index: Option<U256>,
    #[serde(default)]
    pub removed: bool,
}