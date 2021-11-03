use ethereum_types::{H256, U256};

use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Eq)]
pub struct Work {
    pub pow_hash: H256,
    pub seed_hash: H256,
    pub target: H256,
    pub number: Option<u64>,
}

impl Serialize for Work {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self.number.as_ref() {
            Some(num) => (
                &self.pow_hash,
                &self.seed_hash,
                &self.target,
                U256::from(*num),
            )
                .serialize(s),
            None => (&self.pow_hash, &self.seed_hash, &self.target).serialize(s),
        }
    }
}