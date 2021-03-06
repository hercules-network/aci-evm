mod aux_schema;

pub use crate::aux_schema::{load_block_hash, load_transaction_metadata};

use ap_consensus::{ConsensusLog, FRONTIER_ENGINE_ID};
use ap_rpc::EthereumRuntimeRPCApi;
use log::*;
use sc_client_api;
use sc_client_api::{backend::AuxStore, BlockOf};
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{well_known_cache_keys::Id as CacheKeyId, HeaderBackend, ProvideCache};
use sp_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, Error as ConsensusError, ImportResult,
};
use sp_runtime::generic::OpaqueDigestItemId;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, One, Zero};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(derive_more::Display, Debug)]
pub enum Error {
    #[display(fmt = "Multiple post-runtime Ethereum blocks, rejecting!")]
    MultiplePostRuntimeLogs,
    #[display(fmt = "Post-runtime Ethereum block not found, rejecting!")]
    NoPostRuntimeLog,
    #[display(fmt = "Cannot access the runtime at genesis, rejecting!")]
    RuntimeApiCallFailed,
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl std::convert::From<Error> for ConsensusError {
    fn from(error: Error) -> ConsensusError {
        ConsensusError::ClientImport(error.to_string())
    }
}

pub struct FrontierBlockImport<B: BlockT, I, C> {
    inner: I,
    client: Arc<C>,
    enabled: bool,
    _marker: PhantomData<B>,
}

impl<Block: BlockT, I: Clone + BlockImport<Block>, C> Clone for FrontierBlockImport<Block, I, C> {
    fn clone(&self) -> Self {
        FrontierBlockImport {
            inner: self.inner.clone(),
            client: self.client.clone(),
            enabled: self.enabled,
            _marker: PhantomData,
        }
    }
}

impl<B, I, C> FrontierBlockImport<B, I, C>
    where
        B: BlockT,
        I: BlockImport<B, Transaction = sp_api::TransactionFor<C, B>> + Send + Sync,
        I::Error: Into<ConsensusError>,
        C: ProvideRuntimeApi<B> + Send + Sync + HeaderBackend<B> + AuxStore + ProvideCache<B> + BlockOf,
        C::Api: EthereumRuntimeRPCApi<B>,
        C::Api: BlockBuilderApi<B>,
{
    pub fn new(inner: I, client: Arc<C>, enabled: bool) -> Self {
        Self {
            inner,
            client,
            enabled,
            _marker: PhantomData,
        }
    }
}

impl<B, I, C> BlockImport<B> for FrontierBlockImport<B, I, C>
    where
        B: BlockT,
        I: BlockImport<B, Transaction = sp_api::TransactionFor<C, B>> + Send + Sync,
        I::Error: Into<ConsensusError>,
        C: ProvideRuntimeApi<B> + Send + Sync + HeaderBackend<B> + AuxStore + ProvideCache<B> + BlockOf,
        C::Api: EthereumRuntimeRPCApi<B>,
        C::Api: BlockBuilderApi<B>,
{
    type Error = ConsensusError;
    type Transaction = sp_api::TransactionFor<C, B>;

    fn check_block(&mut self, block: BlockCheckParams<B>) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).map_err(Into::into)
    }

    fn import_block(
        &mut self,
        mut block: BlockImportParams<B, Self::Transaction>,
        new_cache: HashMap<CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        macro_rules! insert_closure {
            () => {
                |insert| {
                    block
                        .auxiliary
                        .extend(insert.iter().map(|(k, v)| (k.to_vec(), Some(v.to_vec()))))
                }
            };
        }

        let client = self.client.clone();

        if self.enabled {
            let log = find_frontier_log::<B>(&block.header)?;
            let hash = block.post_hash();
            match log {
                ConsensusLog::EndBlock {
                    block_hash,
                    transaction_hashes,
                } => {
                    let res = aux_schema::write_block_hash(
                        client.as_ref(),
                        block_hash,
                        hash,
                        insert_closure!(),
                    );
                    if res.is_err() {
                        trace!(target: "frontier-consensus", "{:?}", res);
                    }

                    for (index, transaction_hash) in transaction_hashes.into_iter().enumerate() {
                        let res = aux_schema::write_transaction_metadata(
                            client.as_ref(),
                            transaction_hash,
                            (block_hash, index as u32),
                            insert_closure!(),
                        );
                        if res.is_err() {
                            trace!(target: "frontier-consensus", "{:?}", res);
                        }
                    }
                }
            }
            // On importing block 1 we also map the genesis block in the auxiliary.
            if block.header.number().clone() == One::one() {
                let id = BlockId::Number(Zero::zero());
                if let Ok(Some(header)) = client.header(id) {
                    let block = self
                        .client
                        .runtime_api()
                        .current_block(&id)
                        .map_err(|_| Error::RuntimeApiCallFailed)?;
                    let block_hash = block.unwrap().header.hash();
                    let res = aux_schema::write_block_hash(
                        client.as_ref(),
                        block_hash,
                        header.hash(),
                        insert_closure!(),
                    );
                    if res.is_err() {
                        trace!(target: "frontier-consensus", "{:?}", res);
                    }
                }
            }
        }

        self.inner
            .import_block(block, new_cache)
            .map_err(Into::into)
    }
}

fn find_frontier_log<B: BlockT>(header: &B::Header) -> Result<ConsensusLog, Error> {
    let mut frontier_log: Option<_> = None;
    for log in header.digest().logs() {
        trace!(target: "frontier-consensus", "Checking log {:?}, looking for ethereum block.", log);
        let log = log.try_to::<ConsensusLog>(OpaqueDigestItemId::Consensus(&FRONTIER_ENGINE_ID));
        match (log, frontier_log.is_some()) {
            (Some(_), true) => return Err(Error::MultiplePostRuntimeLogs),
            (Some(log), false) => frontier_log = Some(log),
            _ => trace!(target: "frontier-consensus", "Ignoring digest not meant for us"),
        }
    }

    Ok(frontier_log.ok_or(Error::NoPostRuntimeLog)?)
}