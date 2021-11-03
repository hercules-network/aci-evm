#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::ConsensusEngineId;
use sp_std::vec::Vec;

pub const FRONTIER_ENGINE_ID: ConsensusEngineId = [b'f', b'r', b'o', b'n'];

#[derive(Decode, Encode, Clone, PartialEq, Eq)]
pub enum ConsensusLog {
    #[codec(index = 1)]
    EndBlock {
        block_hash: H256,
        transaction_hashes: Vec<H256>,
    },
}