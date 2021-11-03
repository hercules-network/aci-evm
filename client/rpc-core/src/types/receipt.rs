use crate::types::Log;
use ethereum_types::{Bloom as H2048, H160, H256, U256, U64};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub transaction_hash: Option<H256>,
    pub transaction_index: Option<U256>,
    pub block_hash: Option<H256>,
    pub from: Option<H160>,
    pub to: Option<H160>,
    pub block_number: Option<U256>,
    pub cumulative_gas_used: U256,
    pub gas_used: Option<U256>,
    pub contract_address: Option<H160>,
    pub logs: Vec<Log>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "root")]
    pub state_root: Option<H256>,
    pub logs_bloom: H2048,
    #[serde(skip_serializing_if = "Option::is_none", rename = "status")]
    pub status_code: Option<U64>,
}