use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use failure::{bail, err_msg};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use ton_block::{Block, BlockIdExt, BlockInfo, CryptoSignature, CryptoSignaturePair, Deserializable, HashmapAugType, MerkleProof, ShardIdent, ShardStateUnsplit, Transaction, ValidatorDescr};
use ton_types::{Cell, UInt256};
use ton_types::Result;

pub(crate) use errors::ErrorCode;

use crate::boc::internal::{deserialize_object_from_base64, deserialize_object_from_boc_bin};
use crate::client::NetworkUID;
use crate::ClientContext;
use crate::encoding::base64_decode;
use crate::error::ClientResult;
use crate::net::{ParamsOfQueryCollection, query_collection};
use crate::proofs::engine::ProofHelperEngineImpl;
use crate::proofs::errors::Error;
use crate::proofs::validators::{calc_subset_for_workchain, check_crypto_signatures};
use crate::utils::json::JsonHelper;

pub mod errors;
mod engine;
mod validators;

#[cfg(test)]
mod tests;
mod json;

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct ProofsConfig {
    /// Cache proofs in the local storage. Default is `true`.
    /// If this value is set to `true`, downloaded proofs and master-chain BOCs are saved into the
    /// persistent local storage (e.g. file system for native environments or browser's IndexedDB
    /// for the web); otherwise all the data is cached only in memory in current client's context
    /// and will be lost after destruction of the client.
    #[serde(
        default = "default_cache_in_local_storage",
        deserialize_with = "deserialize_cache_in_local_storage"
    )]
    pub cache_in_local_storage: bool,
}

fn default_cache_in_local_storage() -> bool {
    true
}

fn deserialize_cache_in_local_storage<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<bool, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or(default_cache_in_local_storage()))
}

impl Default for ProofsConfig {
    fn default() -> Self {
        Self {
            cache_in_local_storage: default_cache_in_local_storage(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfProofBlockData {
    /// Single block's data, retrieved from TONOS API, that needs proof.
    /// Required fields are `id` and/or top-level `boc` (for block identification), others are
    /// optional.
    pub block: Value,
}

/// Proves that a given block's data, which is queried from TONOS API, can be trusted.
/// 
/// This function checks block proofs and compares given data with the proven.
/// If the given data differs from the proven, the exception will be thrown.
/// The input param is a single block's JSON object, which was queried from DApp server using
/// functions such as `net.query`, `net.query_collection` or `net.wait_for_collection`.
/// If block's BOC is not provided in the JSON, it will be queried from DApp server
/// (in this case it is required to provide at least `id` of block).
///
/// Please note, that joins (like `signatures` in `Block`) are separated entities and not supported,
/// so function will throw an exception in a case if JSON being checked has such entities in it.
///
/// If `cache_in_local_storage` in config is set to `true` (default), downloaded proofs and
/// master-chain BOCs are saved into the persistent local storage (e.g. file system for native
/// environments or browser's IndexedDB for the web); otherwise all the data is cached only in
/// memory in current client's context and will be lost after destruction of the client.
///
/// **Why Proofs are needed**
///
/// Proofs are needed to ensure that the data downloaded from a DApp server is real blockchain
/// data. Checking proofs can protect from the malicious DApp server which can potentially provide
/// fake data, or also from "Man in the Middle" attacks class.
///
/// **What Proofs are**
///
/// Simply, proof is a list of signatures of validators', which have signed this particular master-
/// block.
///
/// The very first validator set's public keys are included in the zero-state. Whe know a root hash
/// of the zero-state, because it is stored in the network configuration file, it is our authority
/// root. For proving zero-state it is enough to calculate and compare its root hash.
///
/// In each new validator cycle the validator set is changed. The new one is stored in a key-block,
/// which is signed by the validator set, which we already trust, the next validator set will be
/// stored to the new key-block and signed by the current validator set, and so on.
///
/// In order to prove any block in the master-chain we need to check, that it has been signed by
/// a trusted validator set. So we need to check all key-blocks' proofs, started from the zero-state
/// and until the block, which we want to prove. But it can take a lot of time and traffic to
/// download and prove all key-blocks on a client. For solving this, special trusted blocks are used
/// in TON-SDK.
///
/// The trusted block is the authority root, as well, as the zero-state. Each trusted block is the
/// `id` (e.g. `root_hash`) of the already proven key-block. There can be plenty of trusted
/// blocks, so there can be a lot of authority roots. The hashes of trusted blocks for MainNet
/// and DevNet are hardcoded in SDK in a separated binary file (trusted_key_blocks.bin) and can 
/// be updated for each release.
/// In future SDK releases, one will also be able to provide their hashes of trusted blocks for
/// other networks, besides for MainNet and DevNet.
/// By using trusted key-blocks, in order to prove any block, we can prove chain of key-blocks to
/// the closest previous trusted key-block, not only to the zero-state.
///
/// But shard-blocks don't have proofs on DApp server. In this case, in order to prove any shard-
/// block data, we search for a corresponding master-block, which contains the root hash of this
/// shard-block, or some shard block which is linked to that block in shard-chain. After proving
/// this master-block, we traverse through each link and calculate and compare hashes with links,
/// one-by-one. After that we can ensure that this shard-block has also been proven.
#[api_function]
pub async fn proof_block_data(
    context: Arc<ClientContext>,
    params: ParamsOfProofBlockData,
) -> ClientResult<()> {
    let engine = ProofHelperEngineImpl::new(context).await
        .map_err(|err| Error::proof_check_failed(err))?;

    let id_opt = params.block["id"].as_str();

    let boc = if let Some(boc) = params.block["boc"].as_str() {
        base64_decode(boc)?
    } else if let Some(id) = id_opt {
        engine.download_block_boc(id).await
            .map_err(|err| Error::proof_check_failed(err))?
    } else {
        return Err(Error::invalid_data("Block's BOC or id are required"));
    };

    let (block, root_hash) = deserialize_object_from_boc_bin(&boc)?;

    engine.proof_block_boc(&root_hash, &block, &boc).await?;

    let block_json = json::serialize_block(root_hash, block, boc)
        .map_err(|err| Error::invalid_data(err))?;

    json::compare_blocks(&params.block, &block_json)
}

#[derive(Serialize, Deserialize, Clone, ApiType, Default)]
pub struct ParamsOfProofTransactionData {
    /// Single transaction's data as queried from DApp server, without modifications.
    /// The required fields are `id` and/or top-level `boc`, others are optional.
    /// In order to reduce network requests count, it is recommended to provide `block_id` and `boc`
    /// of transaction.
    pub transaction: Value,
}

/// Proves that a given transaction's data, which is queried from TONOS API, can be trusted.
///
/// This function requests the corresponding block, checks block proofs, ensures that given transaction
/// exists in the proven block and compares given data with the proven.
/// If the given data differs from the proven, the exception will be thrown.
/// The input parameter is a single transaction's JSON object (see params description), 
/// which was queried from TONOS API using functions such as `net.query`, `net.query_collection` 
/// or `net.wait_for_collection`.
/// 
/// If transaction's BOC and/or `block_id` are not provided in the JSON, they will be queried from
/// TONOS API (in this case it is required to provide at least `id` of transaction).
///
/// Please note, that joins (like `account`, `in_message`, `out_messages`, etc. in `Transaction`
/// entity) are separated entities and not supported, so function will throw an exception in a case
/// if JSON being checked has such entities in it.
///
/// If `cache_in_local_storage` in config is set to `true` (default), downloaded proofs and
/// master-chain BOCs are saved into the persistent local storage (e.g. file system for native
/// environments or browser's IndexedDB for the web); otherwise all the data is cached only in
/// memory in current client's context and will be lost after destruction of the client.
///
/// For more information about proofs checking, see description of `proof_block_data` function.
#[api_function]
pub async fn proof_transaction_data(
    context: Arc<ClientContext>,
    params: ParamsOfProofTransactionData,
) -> ClientResult<()> {
    let engine = ProofHelperEngineImpl::new(context).await
        .map_err(|err| Error::proof_check_failed(err))?;

    let (root_hash, block_id, boc, transaction) =
        transaction_get_required_data(&engine, &params.transaction).await?;

    let block_boc = engine.download_block_boc(&block_id).await
        .map_err(|err| Error::invalid_data(err))?;

    let (block, block_id) = deserialize_object_from_boc_bin(&block_boc)?;

    engine.proof_block_boc(&block_id, &block, &block_boc).await?;

    let block_info = block.read_info()
        .map_err(|err| Error::invalid_data(err))?;
    let block_extra = block.read_extra()
        .map_err(|err| Error::invalid_data(err))?;
    let account_blocks = block_extra.read_account_blocks()
        .map_err(|err| Error::invalid_data(err))?;

    let mut transaction_found_in_block = false;
    account_blocks.iterate_objects(|account_block| {
        account_block.transaction_iterate_full(|_key, cell, _cc| {
            if root_hash == cell.repr_hash() {
                transaction_found_in_block = true;
                return Ok(false);
            }
            Ok(true)
        })
    })
        .map_err(|err| Error::internal_error(err))?;

    if !transaction_found_in_block {
        return Err(Error::proof_check_failed(
            format!(
                "Transaction with `id`: {} not found in block with `id`: {}",
                root_hash.as_hex_string(),
                block_id.as_hex_string(),
            )
        ));
    }

    let transaction_json = json::serialize_transaction(
        root_hash,
        transaction,
        block_id,
        block_info.shard().workchain_id(),
        boc,
    ).map_err(|err| Error::invalid_data(err))?;

    json::compare_transactions(&params.transaction, &transaction_json)
}

async fn transaction_get_required_data<'trans>(
    engine: &ProofHelperEngineImpl,
    transaction_json: &'trans Value,
) -> ClientResult<(UInt256, Cow<'trans, str>, Vec<u8>, Transaction)> {
    let id_opt = Cow::Borrowed(&transaction_json["id"]);
    let mut block_id_opt = transaction_json["block_id"].as_str()
        .map(|str| Cow::Borrowed(str));
    let mut boc_opt = Cow::Borrowed(&transaction_json["boc"]);

    if id_opt.is_null() && boc_opt.is_null() {
        return Err(Error::invalid_data("Transaction's BOC or id are required"));
    }

    if let Some(id) = id_opt.as_str() {
        let mut fields = Vec::new();
        if boc_opt.is_null() {
            fields.push("boc");
        }
        if block_id_opt.is_none() {
            fields.push("block_id");
        }

        if fields.len() > 0 {
            let mut transaction_json = engine.query_transaction_data(id, &fields.join(" ")).await
                .map_err(|err| Error::proof_check_failed(err))?;
            if boc_opt.is_null() {
                boc_opt = Cow::Owned(transaction_json["boc"].take());
            }
            if block_id_opt.is_none() {
                block_id_opt = transaction_json["block_id"].take_string()
                    .map(|string| Cow::Owned(string));
            }
        }
    }

    let transaction_stuff = if let Value::String(boc_base64) = boc_opt.as_ref() {
        deserialize_object_from_base64(boc_base64, "transaction")?
    } else {
        return Err(Error::internal_error("BOC is not found"));
    };

    let root_hash = transaction_stuff.cell.repr_hash();

    let block_id = if let Some(block_id) = block_id_opt {
        block_id
    } else {
        let mut transaction_json = engine.query_transaction_data(
            &root_hash.as_hex_string(),
            "block_id",
        ).await
            .map_err(|err| Error::proof_check_failed(err))?;
        if let Some(block_id) = transaction_json["block_id"].take_string() {
            Cow::Owned(block_id)
        } else {
            return Err(Error::invalid_data("block_id is not found"));
        }
    };

    Ok((
        root_hash,
        block_id,
        transaction_stuff.boc.bytes("transaction")?,
        transaction_stuff.object,
    ))
}

lazy_static! {
    pub(crate) static ref INITIAL_TRUSTED_KEY_BLOCKS: HashMap<[u8; 32], Vec<(u32, [u8; 32])>> =
        bincode::deserialize(include_bytes!("trusted_key_blocks.bin"))
            .expect("FATAL: failed to read trusted key-blocks binary file!");
}

pub(crate) struct Signatures {
    validator_list_hash_short: u32,
    catchain_seqno: u32,
    sig_weight: u64,
    pure_signatures: Vec<CryptoSignaturePair>,
}

impl Signatures {
    pub fn validator_list_hash_short(&self) -> u32 {
        self.validator_list_hash_short
    }

    pub fn catchain_seqno(&self) -> u32 {
        self.catchain_seqno
    }

    pub fn sig_weight(&self) -> u64 {
        self.sig_weight
    }

    pub fn pure_signatures(&self) -> &Vec<CryptoSignaturePair> {
        &self.pure_signatures
    }
}

pub(crate) struct BlockProof {
    id: BlockIdExt,
    root: Cell,
    signatures: Signatures,
}

impl BlockProof {
    pub fn from_value(value: &Value) -> Result<Self> {
        let workchain_id = value.get_i32("workchain_id")?;
        let shard_prefix_tagged = u64::from_str_radix(value.get_str("shard")?, 16)?;
        let shard_id = ShardIdent::with_tagged_prefix(workchain_id, shard_prefix_tagged)?;
        let seq_no = value.get_u32("seq_no")?;
        let root_hash = UInt256::from_str(value.get_str("id")?)?;
        let file_hash = UInt256::from_str(value.get_str("file_hash")?)?;
        let id = BlockIdExt::with_params(shard_id, seq_no, root_hash, file_hash);

        let signatures_json = &value["signatures"];
        let root_boc = base64::decode(signatures_json.get_str("proof")?)?;

        let root = ton_types::deserialize_tree_of_cells(&mut Cursor::new(&root_boc))?;

        let mut pure_signatures = Vec::new();
        let signatures_json_vec = signatures_json.get_array("signatures")?;
        for signature in signatures_json_vec {
            let node_id_short = UInt256::from_str(signature.get_str("node_id")?)?;
            let sign = CryptoSignature::from_r_s_str(
                signature.get_str("r")?,
                signature.get_str("s")?,
            )?;

            pure_signatures.push(CryptoSignaturePair::with_params(node_id_short, sign))
        }

        let signatures = Signatures {
            validator_list_hash_short: signatures_json.get_u32("validator_list_hash_short")?,
            catchain_seqno: signatures_json.get_u32("catchain_seqno")?,
            sig_weight: u64::from_str_radix(
                signatures_json.get_str("sig_weight")?
                    .trim_start_matches("0x"),
                16,
            )?,
            pure_signatures,
        };

        Ok(Self { id, root, signatures })
    }

    #[cfg(test)]
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let proof = ton_block::BlockProof::construct_from_bytes(data)?;
        let signatures = proof.signatures
            .ok_or_else(|| err_msg("Signatures must be filled"))?;

        let mut pure_signatures = Vec::new();
        ton_types::HashmapType::iterate_slices(signatures.pure_signatures.signatures(),
            |ref mut _key, ref mut slice| {
                pure_signatures.push(CryptoSignaturePair::construct_from(slice)?);
                Ok(true)
            })?;

        Ok(Self {
            id: proof.proof_for,
            root: proof.root,
            signatures: Signatures {
                validator_list_hash_short: signatures.validator_info.validator_list_hash_short,
                catchain_seqno: signatures.validator_info.catchain_seqno,
                sig_weight: signatures.pure_signatures.weight(),
                pure_signatures,
            }
        })
    }

    #[cfg(test)]
    pub fn read_from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Self::deserialize(&std::fs::read(path)?)
    }

    pub fn id(&self) -> &BlockIdExt {
        &self.id
    }

    pub fn virtualize_block(&self) -> Result<(Block, Cell)> {
        let merkle_proof = MerkleProof::construct_from(&mut self.root.clone().into())?;
        let block_virt_root = merkle_proof.proof.clone().virtualize(1);
        if *self.id().root_hash() != block_virt_root.repr_hash() {
            bail!(
                "merkle proof has invalid virtual hash (found: {}, expected: {})",
                block_virt_root.repr_hash(),
                self.id(),
            )
        }
        if block_virt_root.repr_hash() != self.id().root_hash {
            bail!(
                "proof for block {} contains a Merkle proof with incorrect root hash: \
                    expected {:x}, found: {:x} ",
                self.id(),
                self.id().root_hash(),
                block_virt_root.repr_hash()
            )
        }
        Ok((Block::construct_from_cell(block_virt_root.clone())?, block_virt_root))
    }

    pub async fn check_proof(&self, engine: &impl ProofHelperEngine) -> Result<(Block, BlockInfo)> {
        if !self.id().shard().is_masterchain() {
            bail!("Only masterchain block proofs are supported");
        }

        let (virt_block, virt_block_info) = self.pre_check_block_proof()?;
        let prev_key_block_seqno = virt_block_info.prev_key_block_seqno();

        if prev_key_block_seqno == 0 {
            let zerostate = engine.load_zerostate().await?;
            self.check_with_zerostate(
                &zerostate,
                &virt_block,
                &virt_block_info,
            )?;
        } else {
            let prev_key_block_proof = engine.load_key_block_proof(prev_key_block_seqno).await?;

            self.check_with_prev_key_block_proof(&prev_key_block_proof, &virt_block, &virt_block_info)?;
        }

        Ok((virt_block, virt_block_info))
    }

    pub fn check_with_prev_key_block_proof(
        &self,
        prev_key_block_proof: &BlockProof,
        virt_block: &Block,
        virt_block_info: &BlockInfo
    ) -> Result<()> {
        if !self.id().shard().is_masterchain() {
            bail!(
                "Can't verify non masterchain block {} using previous key masterchain block",
                self.id(),
            )
        }
        if !prev_key_block_proof.id().shard().is_masterchain() {
            bail!(
                "Invalid previous key block: it's id {} doesn't belong to the masterchain",
                prev_key_block_proof.id(),
            )
        }
        let prev_key_block_seqno = virt_block.read_info()?.prev_key_block_seqno();
        if prev_key_block_proof.id().seq_no as u32 != prev_key_block_seqno {
            bail!(
                "Can't verify block {} using key block {} because the block declares different \
                    previous key block seqno {}",
                self.id(),
                prev_key_block_proof.id(),
                prev_key_block_seqno,
            )
        }
        if prev_key_block_proof.id().seq_no >= self.id().seq_no {
            bail!(
                "Can't verify block {} using key block {} with larger or equal seqno",
                self.id(),
                prev_key_block_proof.id(),
            )
        }
        let (validators, validators_hash_short) =
            self.process_prev_key_block_proof(prev_key_block_proof, virt_block_info.gen_utime().0)?;

        if virt_block_info.key_block() {
            self.pre_check_key_block_proof(virt_block)?;
        }

        self.check_signatures(validators, validators_hash_short)
    }

    fn check_with_zerostate(
        &self,
        zerostate: &ShardStateUnsplit,
        virt_block: &Block,
        virt_block_info: &BlockInfo,
    ) -> Result<()> {
        if virt_block_info.key_block() {
            self.pre_check_key_block_proof(&virt_block)?;
        }

        let (validators, validators_hash_short) =
            self.process_zerostate(zerostate, virt_block_info)?;

        self.check_signatures(validators, validators_hash_short)
    }

    fn pre_check_block_proof(&self) -> Result<(Block, BlockInfo)> {
        let (virt_block, _virt_block_root) = self.virtualize_block()?;

        let info = virt_block.read_info()?;
        let _value_flow = virt_block.read_value_flow()?;
        let _state_update = virt_block.read_state_update()?;

        if info.version() != 0 {
            bail!(
                "proof for block {} contains a Merkle proof with incorrect block info's version {}",
                self.id(),
                info.version(),
            )
        }

        if info.seq_no() != self.id().seq_no() {
            bail!(
                "proof for block {} contains a Merkle proof with seq_no {}, but {} is expected",
                self.id(),
                info.seq_no(),
                self.id().seq_no(),
            )
        }

        if info.shard() != self.id().shard() {
            bail!(
                "proof for block {} contains a Merkle proof with shard id {}, but {} is expected",
                self.id(),
                info.shard(),
                self.id().shard(),
            )
        }

        if info.read_master_ref()?.is_some() != (!info.shard().is_masterchain()) {
            bail!(
                "proof for block {} contains a Merkle proof with invalid not_master flag \
                    in block info",
                self.id(),
            )
        }

        if self.id().shard().is_masterchain() && (info.after_merge() || info.before_split() || info.after_split()) {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares \
                    split/merge for a masterchain block",
                self.id(),
            )
        }

        if info.after_merge() && info.after_split() {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares both \
                    after merge and after split flags",
                self.id(),
            )
        }

        if info.after_split() && (info.shard().is_full()) {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares both \
                    after_split flag and non zero shard prefix",
                self.id(),
            )
        }

        if info.after_merge() && !info.shard().can_split() {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares both \
                    after_merge flag and shard prefix which can't split anymore",
                self.id(),
            )
        }

        if info.key_block() && !self.id().shard().is_masterchain() {
            bail!(
                "proof for block {} contains a Merkle proof which declares non master chain but \
                    key block",
                self.id(),
            )
        }

        Ok((virt_block, info))
    }

    fn pre_check_key_block_proof(&self, virt_block: &Block) -> Result<()> {
        let extra = virt_block.read_extra()?;
        let mc_extra = extra.read_custom()?
            .ok_or_else(|| Error::invalid_data(format!(
                "proof for key block {} contains a Merkle proof without masterchain block extra",
                self.id(),
            )))?;
        let config = mc_extra.config()
            .ok_or_else(|| Error::invalid_data(format!(
                "proof for key block {} contains a Merkle proof without config params",
                self.id(),
            )))?;
        let _cur_validator_set = config.config(34)?
            .ok_or_else(|| Error::invalid_data(format!(
                "proof for key block {} contains a Merkle proof without current validators config \
                    param (34)",
                self.id(),
            )))?;
        for i_config in 32..=38 {
            let _val_set = config.config(i_config)?;
        }
        let _catchain_config = config.config(28)?;

        Ok(())
    }

    fn process_prev_key_block_proof(
        &self,
        prev_key_block_proof: &BlockProof,
        gen_utime: u32
    ) -> Result<(Vec<ValidatorDescr>, u32)> {
        let (virt_key_block, prev_key_block_info) = prev_key_block_proof.pre_check_block_proof()?;

        if !prev_key_block_info.key_block() {
            bail!(
                "proof for key block {} contains a Merkle proof which declares non key block",
                prev_key_block_proof.id(),
            )
        }

        let (validator_set, cc_config) = virt_key_block.read_cur_validator_set_and_cc_conf()
            .map_err(|err| {
                Error::invalid_data(format!(
                    "While checking proof for {}: can't extract config params from key block's \
                        proof {}: {}",
                    self.id(),
                    prev_key_block_proof.id(),
                    err,
                ))
            })?;

        let config = virt_key_block
            .read_extra()?
            .read_custom()?
            .and_then(|custom| custom.config().cloned())
            .ok_or_else(|| err_msg("State doesn't contain `custom` field"))?;
        calc_subset_for_workchain(
            &validator_set,
            &config,
            &cc_config,
            self.id().shard().shard_prefix_with_tag(),
            self.id().shard().workchain_id(),
            self.signatures.catchain_seqno(),
            gen_utime.into()
        )
    }

    fn check_signatures(&self, validators_list: Vec<ValidatorDescr>, list_hash_short: u32) -> Result<()> {
        // Pre checks
        if self.signatures.validator_list_hash_short() != list_hash_short {
            bail!(
                "Bad validator set hash in proof for block {}, calculated: {}, found: {}",
                self.id(),
                list_hash_short,
                self.signatures.validator_list_hash_short(),
            );
        }
        // Check signatures
        let checked_data = ton_block::Block::build_data_for_sign(
            &self.id().root_hash(),
            &self.id().file_hash()
        );
        let total_weight: u64 = validators_list.iter().map(|v| v.weight).sum();
        let weight = check_crypto_signatures(
            &self.signatures,
            &validators_list,
            &checked_data,
        )
            .map_err(|err| {
                Error::invalid_data(
                    format!("Proof for {}: error while check signatures: {}", self.id(), err)
                )
            })?;

        // Check weight
        if weight != self.signatures.sig_weight() {
            bail!(
                "Proof for {}: total signature weight mismatch: declared: {}, calculated: {}",
                self.id(),
                self.signatures.sig_weight(),
                weight,
            );
        }

        if weight * 3 <= total_weight * 2 {
            bail!(
                "Proof for {}: too small signatures weight",
                self.id(),
            );
        }

        Ok(())
    }

    fn process_zerostate(
        &self,
        zerostate: &ShardStateUnsplit,
        block_info: &ton_block::BlockInfo,
    ) -> Result<(Vec<ValidatorDescr>, u32)> {
        if !self.id().shard().is_masterchain() {
            bail!(
                "Can't check proof for non master block {} using master state",
                self.id(),
            );
        }
        if block_info.prev_key_block_seqno() > 0 {
            bail!(
                "Can't check proof for block {} using zerostate, because it is older than \
                    the previous key block with seq_no {}",
                self.id(),
                block_info.prev_key_block_seqno(),
            );
        }

        let (cur_validator_set, cc_config) = zerostate.read_cur_validator_set_and_cc_conf()?;
        let mc_state_extra = zerostate.read_custom()?
            .ok_or_else(|| err_msg("Can't read custom field from the zerostate"))?;

        let (validators, hash_short) = calc_subset_for_workchain(
            &cur_validator_set,
            mc_state_extra.config(),
            &cc_config,
            self.id().shard().shard_prefix_with_tag(),
            self.id().shard().workchain_id(),
            self.signatures.catchain_seqno(),
            block_info.gen_utime()
        )?;

        Ok((validators, hash_short))
    }
}

async fn get_current_network_uid(
    context: &Arc<ClientContext>,
) -> Result<Arc<NetworkUID>> {
    if let Some(ref uid) = *context.net.network_uid.read().await {
        return Ok(Arc::clone(uid));
    }

    let queried_uid = query_current_network_uid(Arc::clone(context)).await?;

    let mut write_guard = context.net.network_uid.write().await;
    if let Some(ref stored_uid) = *write_guard {
        return Ok(Arc::clone(stored_uid));
    }

    *write_guard = Some(Arc::clone(&queried_uid));

    Ok(queried_uid)
}

async fn query_current_network_uid(
    context: Arc<ClientContext>,
) -> Result<Arc<NetworkUID>> {
    let blocks = query_collection(context, ParamsOfQueryCollection {
        collection: "blocks".to_string(),
        filter: Some(json!({
            "workchain_id": {
                "eq": -1
            },
            "seq_no": {
                "eq": 1
            },
        })),
        result: "id, prev_ref{root_hash}".to_string(),
        limit: Some(1),
        ..Default::default()
    }).await?.result;

    if blocks.is_empty() {
        bail!("Unable to resolve zerostate's root hash: can't get masterchain block #1");
    }

    let prev_ref = &blocks[0]["prev_ref"];
    if prev_ref.is_null() {
        bail!("Unable to resolve zerostate's root hash: prev_ref of the block #1 is not set");
    }

    let first_master_block_root_hash = UInt256::from_str(blocks[0].get_str("id")?)?;
    let zerostate_root_hash = UInt256::from_str(prev_ref.get_str("root_hash")?)?;

    Ok(Arc::new(NetworkUID { zerostate_root_hash, first_master_block_root_hash }))
}

async fn resolve_initial_trusted_key_block(
    context: &Arc<ClientContext>,
    mc_seq_no: u32,
) -> Result<(u32, UInt256)> {
    let network_uid = get_current_network_uid(context).await?;

    if let Some(hardcoded_mc_blocks) =
        INITIAL_TRUSTED_KEY_BLOCKS.get(network_uid.zerostate_root_hash.as_array())
    {
        let index = match hardcoded_mc_blocks.binary_search_by_key(
            &mc_seq_no, |(seq_no, _root_hash)| *seq_no,
        ) {
            Ok(seq_no) => seq_no,
            Err(seq_no) => if seq_no >= hardcoded_mc_blocks.len() {
                seq_no - 1
            } else {
                seq_no
            },
        };

        let (seq_no, ref root_hash) = hardcoded_mc_blocks[index];
        return Ok((seq_no, UInt256::from_slice(root_hash)));
    }

    bail!(
        "Unable to resolve trusted key-block for network with zerostate root_hash: `{}`",
        network_uid.zerostate_root_hash,
    )
}

#[async_trait::async_trait]
pub(crate) trait ProofHelperEngine {
    async fn load_zerostate(&self) -> Result<ShardStateUnsplit>;
    async fn load_key_block_proof(&self, mc_seq_no: u32) -> Result<BlockProof>;
}
