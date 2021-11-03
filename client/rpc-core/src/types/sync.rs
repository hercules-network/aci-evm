use std::collections::BTreeMap;

use ethereum_types::{H512, U256};
use serde::{Serialize, Serializer};

#[derive(Default, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SyncInfo {
    pub starting_block: U256,
    pub current_block: U256,
    pub highest_block: U256,
    pub warp_chunks_amount: Option<U256>,
    pub warp_chunks_processed: Option<U256>,
}

#[derive(Default, Debug, Serialize)]
pub struct Peers {
    pub active: usize,
    pub connected: usize,
    pub max: u32,
    pub peers: Vec<PeerInfo>,
}

#[derive(Default, Debug, Serialize)]
pub struct PeerInfo {
    pub id: Option<String>,
    pub name: String,
    pub caps: Vec<String>,
    pub network: PeerNetworkInfo,
    pub protocols: PeerProtocolsInfo,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerNetworkInfo {
    pub remote_address: String,
    pub local_address: String,
}

#[derive(Default, Debug, Serialize)]
pub struct PeerProtocolsInfo {
    pub eth: Option<EthProtocolInfo>,
    pub pip: Option<PipProtocolInfo>,
}

#[derive(Default, Debug, Serialize)]
pub struct EthProtocolInfo {
    pub version: u32,
    pub difficulty: Option<U256>,
    pub head: String,
}

#[derive(Default, Debug, Serialize)]
pub struct PipProtocolInfo {
    pub version: u32,
    pub difficulty: U256,
    pub head: String,
}

#[derive(Debug, PartialEq)]
pub enum SyncStatus {
    Info(SyncInfo),
    None,
}

impl Serialize for SyncStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match *self {
            SyncStatus::Info(ref info) => info.serialize(serializer),
            SyncStatus::None => false.serialize(serializer),
        }
    }
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStats {
    pub first_seen: u64,
    pub propagated_to: BTreeMap<H512, usize>,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainStatus {
    pub block_gap: Option<(U256, U256)>,
}