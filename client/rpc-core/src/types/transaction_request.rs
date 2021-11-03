use crate::types::Bytes;
use ethereum_types::{H160, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub from: Option<H160>,
    pub to: Option<H160>,
    pub gas_price: Option<U256>,
    pub gas: Option<U256>,
    pub value: Option<U256>,
    pub data: Option<Bytes>,
    pub nonce: Option<U256>,
}