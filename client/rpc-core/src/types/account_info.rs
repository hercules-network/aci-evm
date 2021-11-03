use crate::types::Bytes;
use ethereum_types::{Address, Public, H160, H256, U256};
use serde::Serialize;

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct AccountInfo {
    pub name: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageProof {
    pub key: U256,
    pub value: U256,
    pub proof: Vec<Bytes>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EthAccount {
    pub address: H160,
    pub balance: U256,
    pub nonce: U256,
    pub code_hash: H256,
    pub storage_hash: H256,
    pub account_proof: Vec<Bytes>,
    pub storage_proof: Vec<StorageProof>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct ExtAccountInfo {
    pub name: String,
    pub meta: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveredAccount {
    pub address: Address,
    pub public_key: Public,
    pub is_valid_for_current_chain: bool,
}