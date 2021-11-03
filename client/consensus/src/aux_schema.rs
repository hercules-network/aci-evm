use codec::{Decode, Encode};
use sc_client_api::backend::AuxStore;
use sp_blockchain::{Error as ClientError, Result as ClientResult};
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;

fn load_decode<B: AuxStore, T: Decode>(backend: &B, key: &[u8]) -> ClientResult<Option<T>> {
    let corrupt = |e: codec::Error| {
        ClientError::Backend(format!("Frontier DB is corrupted. Decode error: {}", e))
    };
    match backend.get_aux(key)? {
        None => Ok(None),
        Some(t) => T::decode(&mut &t[..]).map(Some).map_err(corrupt),
    }
}

pub fn block_hash_key(ethereum_block_hash: H256) -> Vec<u8> {
    let mut ret = b"ethereum_block_hash:".to_vec();
    ret.append(&mut ethereum_block_hash.as_ref().to_vec());
    ret
}

pub fn load_block_hash<Block: BlockT, B: AuxStore>(
    backend: &B,
    hash: H256,
) -> ClientResult<Option<Vec<Block::Hash>>> {
    let key = block_hash_key(hash);
    load_decode(backend, &key)
}

pub fn write_block_hash<Hash: Encode + Decode, F, R, Backend: AuxStore>(
    client: &Backend,
    ethereum_hash: H256,
    block_hash: Hash,
    write_aux: F,
) -> ClientResult<R>
    where
        F: FnOnce(&[(&[u8], &[u8])]) -> R,
{
    let key = block_hash_key(ethereum_hash);
    let mut data: Vec<Hash> = match load_decode(client, &key) {
        Ok(Some(hashes)) => hashes,
        Ok(None) => Vec::new(),
        Err(e) => return Err(e),
    };
    data.push(block_hash);
    Ok(write_aux(&[(&key, &data.encode()[..])]))
}

pub fn transaction_metadata_key(ethereum_transaction_hash: H256) -> Vec<u8> {
    let mut ret = b"ethereum_transaction_hash:".to_vec();
    ret.append(&mut ethereum_transaction_hash.as_ref().to_vec());
    ret
}

pub fn load_transaction_metadata<B: AuxStore>(
    backend: &B,
    hash: H256,
) -> ClientResult<Option<Vec<(H256, u32)>>> {
    let key = transaction_metadata_key(hash);
    load_decode(backend, &key)
}

pub fn write_transaction_metadata<F, R, Backend: AuxStore>(
    client: &Backend,
    hash: H256,
    metadata: (H256, u32),
    write_aux: F,
) -> ClientResult<R>
    where
        F: FnOnce(&[(&[u8], &[u8])]) -> R,
{
    let key = transaction_metadata_key(hash);
    let mut data: Vec<(H256, u32)> = match load_decode(client, &key) {
        Ok(Some(metadata)) => metadata,
        Ok(None) => Vec::new(),
        Err(e) => return Err(e),
    };
    data.push(metadata);
    Ok(write_aux(&[(&key, &data.encode()[..])]))
}