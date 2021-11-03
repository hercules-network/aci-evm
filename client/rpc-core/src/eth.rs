use ethereum_types::{H160, H256, H64, U256, U64};
use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_derive::rpc;

use crate::types::{
    BlockNumber, Bytes, CallRequest, Filter, FilterChanges, Index, Log, Receipt, RichBlock,
    SyncStatus, Transaction, TransactionRequest, Work,
};
pub use rpc_impl_EthApi::gen_server::EthApi as EthApiServer;
pub use rpc_impl_EthFilterApi::gen_server::EthFilterApi as EthFilterApiServer;

#[rpc(server)]
pub trait EthApi {
    #[rpc(name = "eth_protocolVersion")]
    fn protocol_version(&self) -> Result<u64>;

    #[rpc(name = "eth_syncing")]
    fn syncing(&self) -> Result<SyncStatus>;

    #[rpc(name = "eth_hashrate")]
    fn hashrate(&self) -> Result<U256>;

    #[rpc(name = "eth_coinbase")]
    fn author(&self) -> Result<H160>;

    #[rpc(name = "eth_mining")]
    fn is_mining(&self) -> Result<bool>;

    #[rpc(name = "eth_chainId")]
    fn chain_id(&self) -> Result<Option<U64>>;

    #[rpc(name = "eth_gasPrice")]
    fn gas_price(&self) -> Result<U256>;

    #[rpc(name = "eth_accounts")]
    fn accounts(&self) -> Result<Vec<H160>>;

    #[rpc(name = "eth_blockNumber")]
    fn block_number(&self) -> Result<U256>;

    #[rpc(name = "eth_getBalance")]
    fn balance(&self, _: H160, _: Option<BlockNumber>) -> Result<U256>;

    #[rpc(name = "eth_getStorageAt")]
    fn storage_at(&self, _: H160, _: U256, _: Option<BlockNumber>) -> Result<H256>;

    #[rpc(name = "eth_getBlockByHash")]
    fn block_by_hash(&self, _: H256, _: bool) -> Result<Option<RichBlock>>;

    #[rpc(name = "eth_getBlockByNumber")]
    fn block_by_number(&self, _: BlockNumber, _: bool) -> Result<Option<RichBlock>>;

    #[rpc(name = "eth_getTransactionCount")]
    fn transaction_count(&self, _: H160, _: Option<BlockNumber>) -> Result<U256>;

    #[rpc(name = "eth_getBlockTransactionCountByHash")]
    fn block_transaction_count_by_hash(&self, _: H256) -> Result<Option<U256>>;

    #[rpc(name = "eth_getBlockTransactionCountByNumber")]
    fn block_transaction_count_by_number(&self, _: BlockNumber) -> Result<Option<U256>>;

    #[rpc(name = "eth_getUncleCountByBlockHash")]
    fn block_uncles_count_by_hash(&self, _: H256) -> Result<U256>;

    #[rpc(name = "eth_getUncleCountByBlockNumber")]
    fn block_uncles_count_by_number(&self, _: BlockNumber) -> Result<U256>;

    #[rpc(name = "eth_getCode")]
    fn code_at(&self, _: H160, _: Option<BlockNumber>) -> Result<Bytes>;

    #[rpc(name = "eth_sendTransaction")]
    fn send_transaction(&self, _: TransactionRequest) -> BoxFuture<H256>;

    #[rpc(name = "eth_sendRawTransaction")]
    fn send_raw_transaction(&self, _: Bytes) -> BoxFuture<H256>;

    #[rpc(name = "eth_call")]
    fn call(&self, _: CallRequest, _: Option<BlockNumber>) -> Result<Bytes>;

    #[rpc(name = "eth_estimateGas")]
    fn estimate_gas(&self, _: CallRequest, _: Option<BlockNumber>) -> Result<U256>;

    #[rpc(name = "eth_getTransactionByHash")]
    fn transaction_by_hash(&self, _: H256) -> Result<Option<Transaction>>;

    #[rpc(name = "eth_getTransactionByBlockHashAndIndex")]
    fn transaction_by_block_hash_and_index(&self, _: H256, _: Index)
                                           -> Result<Option<Transaction>>;

    #[rpc(name = "eth_getTransactionByBlockNumberAndIndex")]
    fn transaction_by_block_number_and_index(
        &self,
        _: BlockNumber,
        _: Index,
    ) -> Result<Option<Transaction>>;

    #[rpc(name = "eth_getTransactionReceipt")]
    fn transaction_receipt(&self, _: H256) -> Result<Option<Receipt>>;

    #[rpc(name = "eth_getUncleByBlockHashAndIndex")]
    fn uncle_by_block_hash_and_index(&self, _: H256, _: Index) -> Result<Option<RichBlock>>;

    #[rpc(name = "eth_getUncleByBlockNumberAndIndex")]
    fn uncle_by_block_number_and_index(
        &self,
        _: BlockNumber,
        _: Index,
    ) -> Result<Option<RichBlock>>;

    #[rpc(name = "eth_getLogs")]
    fn logs(&self, _: Filter) -> Result<Vec<Log>>;

    #[rpc(name = "eth_getWork")]
    fn work(&self) -> Result<Work>;

    #[rpc(name = "eth_submitWork")]
    fn submit_work(&self, _: H64, _: H256, _: H256) -> Result<bool>;

    #[rpc(name = "eth_submitHashrate")]
    fn submit_hashrate(&self, _: U256, _: H256) -> Result<bool>;
}

#[rpc(server)]
pub trait EthFilterApi {
    #[rpc(name = "eth_newFilter")]
    fn new_filter(&self, _: Filter) -> Result<U256>;

    #[rpc(name = "eth_newBlockFilter")]
    fn new_block_filter(&self) -> Result<U256>;

    #[rpc(name = "eth_newPendingTransactionFilter")]
    fn new_pending_transaction_filter(&self) -> Result<U256>;

    #[rpc(name = "eth_getFilterChanges")]
    fn filter_changes(&self, _: Index) -> Result<FilterChanges>;

    #[rpc(name = "eth_getFilterLogs")]
    fn filter_logs(&self, _: Index) -> Result<Vec<Log>>;

    #[rpc(name = "eth_uninstallFilter")]
    fn uninstall_filter(&self, _: Index) -> Result<bool>;
}