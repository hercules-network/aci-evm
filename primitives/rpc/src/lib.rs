#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use ethereum::{Block as EthereumBlock, Log};
use ethereum_types::Bloom;
use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;

#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_runtime::RuntimeDebug)]
pub struct TransactionStatus {
    pub transaction_hash: H256,
    pub transaction_index: u32,
    pub from: H160,
    pub to: Option<H160>,
    pub contract_address: Option<H160>,
    pub logs: Vec<Log>,
    pub logs_bloom: Bloom,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus {
            transaction_hash: H256::default(),
            transaction_index: 0 as u32,
            from: H160::default(),
            to: None,
            contract_address: None,
            logs: Vec::new(),
            logs_bloom: Bloom::default(),
        }
    }
}

sp_api::decl_runtime_apis! {
    pub trait EthereumRuntimeRPCApi {
        fn chain_id() -> u64;
        fn account_basic(address: H160) -> ap_evm::Account;
        fn gas_price() -> U256;
        fn account_code_at(address: H160) -> Vec<u8>;
        fn author() -> H160;
        fn storage_at(address: H160, index: U256) -> H256;
        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            gas_price: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
        ) -> Result<ap_evm::CallInfo, sp_runtime::DispatchError>;
        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            gas_price: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
        ) -> Result<ap_evm::CreateInfo, sp_runtime::DispatchError>;
        fn current_block() -> Option<EthereumBlock>;
        fn current_receipts() -> Option<Vec<ethereum::Receipt>>;
        fn current_transaction_statuses() -> Option<Vec<TransactionStatus>>;
        fn current_all() -> (
            Option<EthereumBlock>,
            Option<Vec<ethereum::Receipt>>,
            Option<Vec<TransactionStatus>>
        );
        fn current_block_gas_limit() -> U256;
    }
}

pub trait ConvertTransaction<E> {
    fn convert_transaction(&self, transaction: ethereum::Transaction) -> E;
}