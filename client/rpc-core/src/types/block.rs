use std::collections::BTreeMap;
use std::ops::Deref;

use crate::types::{Bytes, Transaction};
use ethereum_types::{Bloom as H2048, H160, H256, U256};
use serde::ser::Error;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum BlockTransactions {
    Hashes(Vec<H256>),
    Full(Vec<Transaction>),
}

impl Serialize for BlockTransactions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match *self {
            BlockTransactions::Hashes(ref hashes) => hashes.serialize(serializer),
            BlockTransactions::Full(ref ts) => ts.serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub hash: Option<H256>,
    pub parent_hash: H256,
    #[serde(rename = "sha3Uncles")]
    pub uncles_hash: H256,
    pub author: H160,
    pub miner: H160,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub number: Option<U256>,
    pub gas_used: U256,
    pub gas_limit: U256,
    pub extra_data: Bytes,
    pub logs_bloom: Option<H2048>,
    pub timestamp: U256,
    pub difficulty: U256,
    pub total_difficulty: Option<U256>,
    pub seal_fields: Vec<Bytes>,
    pub uncles: Vec<H256>,
    pub transactions: BlockTransactions,
    pub size: Option<U256>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub hash: Option<H256>,
    pub parent_hash: H256,
    #[serde(rename = "sha3Uncles")]
    pub uncles_hash: H256,
    pub author: H160,
    pub miner: H160,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub number: Option<U256>,
    pub gas_used: U256,
    pub gas_limit: U256,
    pub extra_data: Bytes,
    pub logs_bloom: H2048,
    pub timestamp: U256,
    pub difficulty: U256,
    pub seal_fields: Vec<Bytes>,
    pub size: Option<U256>,
}

pub type RichBlock = Rich<Block>;

pub type RichHeader = Rich<Header>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rich<T> {
    pub inner: T,
    pub extra_info: BTreeMap<String, String>,
}

impl<T> Deref for Rich<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Serialize> Serialize for Rich<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        use serde_json::{to_value, Value};
        let serialized = (to_value(&self.inner), to_value(&self.extra_info));
        if let (Ok(Value::Object(mut value)), Ok(Value::Object(extras))) = serialized {
            value.extend(extras);
            value.serialize(serializer)
        } else {
            Err(S::Error::custom(
                "Unserializable structures: expected objects",
            ))
        }
    }
}