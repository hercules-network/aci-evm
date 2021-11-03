use crate::types::Bytes;
use ethereum_types::{H160, H256, H512, U256, U64};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: H256,
    pub nonce: U256,
    pub block_hash: Option<H256>,
    pub block_number: Option<U256>,
    pub transaction_index: Option<U256>,
    pub from: H160,
    pub to: Option<H160>,
    pub value: U256,
    pub gas_price: U256,
    pub gas: U256,
    pub input: Bytes,
    pub creates: Option<H160>,
    pub raw: Bytes,
    pub public_key: Option<H512>,
    pub chain_id: Option<U64>,
    pub standard_v: U256,
    pub v: U256,
    pub r: U256,
    pub s: U256,
}

#[derive(Debug)]
pub enum LocalTransactionStatus {
    Pending,
    Future,
    Mined(Transaction),
    Culled(Transaction),
    Dropped(Transaction),
    Replaced(Transaction, U256, H256),
    Rejected(Transaction, String),
    Invalid(Transaction),
    Canceled(Transaction),
}

impl Serialize for LocalTransactionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        use self::LocalTransactionStatus::*;
        let elems = match *self {
            Pending | Future => 1,
            Mined(..) | Culled(..) | Dropped(..) | Invalid(..) | Canceled(..) => 2,
            Rejected(..) => 3,
            Replaced(..) => 4,
        };
        let status = "status";
        let transaction = "transaction";
        let mut struc = serializer.serialize_struct("LocalTransactionStatus", elems)?;
        match *self {
            Pending => struc.serialize_field(status, "pending")?,
            Future => struc.serialize_field(status, "future")?,
            Mined(ref tx) => {
                struc.serialize_field(status, "mined")?;
                struc.serialize_field(transaction, tx)?;
            }
            Culled(ref tx) => {
                struc.serialize_field(status, "culled")?;
                struc.serialize_field(transaction, tx)?;
            }
            Dropped(ref tx) => {
                struc.serialize_field(status, "dropped")?;
                struc.serialize_field(transaction, tx)?;
            }
            Canceled(ref tx) => {
                struc.serialize_field(status, "canceled")?;
                struc.serialize_field(transaction, tx)?;
            }
            Invalid(ref tx) => {
                struc.serialize_field(status, "invalid")?;
                struc.serialize_field(transaction, tx)?;
            }
            Rejected(ref tx, ref reason) => {
                struc.serialize_field(status, "rejected")?;
                struc.serialize_field(transaction, tx)?;
                struc.serialize_field("error", reason)?;
            }
            Replaced(ref tx, ref gas_price, ref hash) => {
                struc.serialize_field(status, "replaced")?;
                struc.serialize_field(transaction, tx)?;
                struc.serialize_field("hash", hash)?;
                struc.serialize_field("gasPrice", gas_price)?;
            }
        }
        struc.end()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct RichRawTransaction {
    pub raw: Bytes,
    #[serde(rename = "tx")]
    pub transaction: Transaction,
}

pub struct PendingTransaction {
    pub transaction: Transaction,
    pub at_block: u64,
}

impl PendingTransaction {
    pub fn new(transaction: Transaction, at_block: u64) -> Self {
        Self {
            transaction,
            at_block,
        }
    }
}

pub type PendingTransactions = Option<Arc<Mutex<HashMap<H256, PendingTransaction>>>>;