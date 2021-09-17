use std::collections::HashMap;
use std::ops::Deref;
#[cfg(test)]
use std::path::Path;
use std::sync::Arc;

use failure::{bail, err_msg};
use ton_block::{
    Block, BlockIdExt, BlockInfo, CatchainConfig, ConfigParams, Deserializable, MerkleProof,
    ShardStateUnsplit, ValidatorDescr, ValidatorSet,
};
use ton_types::{Cell, deserialize_tree_of_cells, HashmapType};
use ton_types::Result;

use crate::ClientContext;
use crate::error::{ClientError, ClientResult};
use crate::net::{ParamsOfQueryCollection, query_collection};
use crate::net::types::TrustedMcBlockId;
use crate::proofs::errors::Error;
use crate::proofs::validators::{calc_subset_for_workchain, check_crypto_signatures};

mod errors;
mod engine;
pub(crate) mod storage;
mod validators;

#[cfg(test)]
mod tests;

// TODO: Update this JSON-file contents using CI:
static INITIAL_TRUSTED_KEY_BLOCKS_JSON: &str = include_str!("trusted_key_blocks.json");

#[derive(serde::Deserialize)]
pub(crate) struct TrustedKeyBlockJsonEntry {
    trusted_key_block: TrustedMcBlockId,
}

lazy_static! {
    pub(crate) static ref INITIAL_TRUSTED_KEY_BLOCKS: HashMap<String, TrustedKeyBlockJsonEntry> =
        serde_json::from_str(INITIAL_TRUSTED_KEY_BLOCKS_JSON)
            .expect("FATAL: failed to parse trusted key-blocks JSON!");
}

pub struct BlockProof(ton_block::BlockProof);

impl Deref for BlockProof {
    type Target = ton_block::BlockProof;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ton_block::BlockProof> for BlockProof {
    fn from(internal: ton_block::BlockProof) -> Self {
        Self(internal)
    }
}

impl BlockProof {
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let root = deserialize_tree_of_cells(&mut std::io::Cursor::new(data))?;
        let internal = ton_block::BlockProof::construct_from(&mut root.clone().into())?;

        Ok(Self(internal))
    }

    #[cfg(test)]
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::deserialize(&std::fs::read(path)?)
    }

    pub fn id(&self) -> &BlockIdExt {
        &self.proof_for
    }

    pub fn is_link(&self) -> bool {
        !self.id().shard().is_masterchain()
    }

    pub fn virtualize_block(&self) -> Result<(Block, Cell)> {
        let merkle_proof = MerkleProof::construct_from(&mut self.root.clone().into())?;
        let block_virt_root = merkle_proof.proof.clone().virtualize(1);
        if *self.proof_for.root_hash() != block_virt_root.repr_hash() {
            bail!(
                "merkle proof has invalid virtual hash (found: {}, expected: {})",
                block_virt_root.repr_hash(),
                self.proof_for,
            )
        }
        if block_virt_root.repr_hash() != self.id().root_hash {
            bail!(
                "proof for block {} contains a Merkle proof with incorrect root hash: expected {:x}, found: {:x} ",
                self.id(),
                self.id().root_hash(),
                block_virt_root.repr_hash()
            )
        }
        Ok((Block::construct_from_cell(block_virt_root.clone())?, block_virt_root))
    }

    pub fn check_with_prev_key_block_proof(&self, prev_key_block_proof: &BlockProof) -> Result<()> {
        let (virt_block, virt_block_info) = self.pre_check_block_proof()?;
        self.check_with_prev_key_block_proof_(prev_key_block_proof, &virt_block, &virt_block_info)?;

        Ok(())
    }

    pub fn check_with_master_state(
        &self,
        block_id: &BlockIdExt,
        master_state: &ShardStateUnsplit,
        config: &ConfigParams
    ) -> Result<()> {
        if self.is_link() {
            bail!(
                "Can't verify block {}: can't call `check_with_master_state` for proof link",
                self.id(),
            )
        }

        let (virt_block, virt_block_info) = self.pre_check_block_proof()?;

        self.check_with_master_state_(block_id, master_state, config, &virt_block, &virt_block_info)?;

        Ok(())
    }

    pub fn check_proof_link(&self) -> Result<()> {
        if !self.is_link() {
            bail!("Can't call `check_proof_link` not for proof link, block {}", self.id())
        }
        self.pre_check_block_proof()?;

        Ok(())
    }

    pub async fn check_proof(&self, engine: &impl ProofHelperEngine) -> Result<()> {
        if self.is_link() {
            self.check_proof_link()?;
        } else {
            let (virt_block, virt_block_info) = self.pre_check_block_proof()?;
            let prev_key_block_seqno = virt_block_info.prev_key_block_seqno();

            if prev_key_block_seqno == 0 {
                let zerostate = engine.load_zerostate().await?;
                let mc_state_extra = zerostate.read_custom()?
                    .ok_or_else(|| err_msg("Can't read custom field from the zerostate"))?;
                self.check_with_master_state_(
                    self.id(),
                    &zerostate,
                    mc_state_extra.config(),
                    &virt_block,
                    &virt_block_info,
                )?;
            } else {
                let prev_key_block_proof = engine.load_key_block_proof(prev_key_block_seqno).await?;

                self.check_with_prev_key_block_proof_(&prev_key_block_proof, &virt_block, &virt_block_info)?;
            }
        }
        Ok(())
    }

    pub fn get_cur_validators_set(&self) -> Result<(ValidatorSet, CatchainConfig)> {
        let (virt_key_block, prev_key_block_info) = self.pre_check_block_proof()?;

        if !prev_key_block_info.key_block() {
            bail!(
                "proof for key block {} contains a Merkle proof which declares non key block",
                self.id(),
            )
        }

        let (cur_validator_set, cc_config) = virt_key_block.read_cur_validator_set_and_cc_conf()
            .map_err(|err| {
                Error::invalid_data(format!(
                    "Сan't extract config params from key block's proof {}: {}",
                    self.id(),
                    err,
                ))
            })?;

        Ok((cur_validator_set, cc_config))
    }

    pub fn check_with_prev_key_block_proof_(
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
                "Can't verify block {} using key block {} because the block declares different previous key block seqno {}",
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

    fn check_with_master_state_(
        &self,
        block_id: &BlockIdExt,
        master_state: &ShardStateUnsplit,
        config: &ConfigParams,
        virt_block: &Block,
        virt_block_info: &BlockInfo,
    ) -> Result<()> {
        if virt_block_info.key_block() {
            self.pre_check_key_block_proof(&virt_block)?;
        }

        let (validators, validators_hash_short) =
            self.process_given_state(block_id, master_state, virt_block_info, config)?;

        self.check_signatures(validators, validators_hash_short)
    }

    fn pre_check_block_proof(&self) -> Result<(Block, BlockInfo)> {
        if !self.id().shard().is_masterchain() && self.signatures.is_some() {
            bail!(
                "proof for non-master block {} can't contain signatures",
                self.id(),
            )
        }

        let (virt_block, virt_block_root) = self.virtualize_block()?;

        if virt_block_root.repr_hash() != self.id().root_hash() {
            bail!(
                "proof for block {} contains a Merkle proof with incorrect root hash: expected {}, found: {} ",
                self.id(),
                self.id().root_hash(),
                virt_block_root.repr_hash(),
            )
        }

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
                "proof for block {} contains a Merkle proof with invalid not_master flag in block info",
                self.id(),
            )
        }

        if self.id().shard().is_masterchain() && (info.after_merge() || info.before_split() || info.after_split()) {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares split/merge for a masterchain block",
                self.id(),
            )
        }

        if info.after_merge() && info.after_split() {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares both after merge and after split flags",
                self.id(),
            )
        }

        if info.after_split() && (info.shard().is_full()) {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares both after_split flag and non zero shard prefix",
                self.id(),
            )
        }

        if info.after_merge() && !info.shard().can_split() {
            bail!(
                "proof for block {} contains a Merkle proof with a block info which declares both after_merge flag and shard prefix which can't split anymore",
                self.id(),
            )
        }

        if info.key_block() && !self.id().shard().is_masterchain() {
            bail!(
                "proof for block {} contains a Merkle proof which declares non master chain but key block",
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
                "proof for key block {} contains a Merkle proof without current validators config param (34)",
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
                    "While checking proof for {}: can't extract config params from key block's proof {}: {}",
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
            self.signatures.as_ref().map(|s| s.validator_info.catchain_seqno).unwrap_or(0),
            gen_utime.into()
        )
    }

    fn check_signatures(&self, validators_list: Vec<ValidatorDescr>, list_hash_short: u32) -> Result<()> {
        // Pre checks
        if self.signatures.is_none() {
            bail!(
                "Proof for {} doesn't have signatures to check",
                self.id(),
            );
        }
        let signatures = self.signatures.as_ref().unwrap();
        if signatures.validator_info.validator_list_hash_short != list_hash_short {
            bail!(
                "Bad validator set hash in proof for block {}, calculated: {}, found: {}",
                self.id(),
                list_hash_short,
                signatures.validator_info.validator_list_hash_short,
            );
        }
        let expected_count = signatures.pure_signatures.count() as usize;
        let count = signatures.pure_signatures.signatures().count(expected_count)?;
        if expected_count != count {
            bail!(
                "Proof for {}: signature count mismatch: declared: {}, calculated: {}",
                self.id(),
                expected_count,
                count,
            );
        }

        // Check signatures
        let checked_data = ton_block::Block::build_data_for_sign(
            &self.id().root_hash(),
            &self.id().file_hash()
        );
        let total_weight: u64 = validators_list.iter().map(|v| v.weight).sum();
        let weight = check_crypto_signatures(&signatures.pure_signatures, &validators_list, &checked_data)
            .map_err(|err| {
                Error::invalid_data(
                    format!("Proof for {}: error while check signatures: {}", self.id(), err)
                )
            })?;

        // Check weight
        if weight != signatures.pure_signatures.weight() {
            bail!(
                "Proof for {}: total signature weight mismatch: declared: {}, calculated: {}",
                self.id(),
                signatures.pure_signatures.weight(),
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

    fn process_given_state(
        &self,
        block_id: &BlockIdExt,
        state: &ShardStateUnsplit,
        block_info: &ton_block::BlockInfo,
        config: &ConfigParams,
    ) -> Result<(Vec<ValidatorDescr>, u32)> {
        // Checks
        if !block_id.shard().is_masterchain() {
            bail!(
                "Can't check proof for {}: given state {} doesn't belong masterchain",
                self.id(),
                block_id,
            );
        }
        if !self.id().shard().is_masterchain() {
            bail!(
                "Can't check proof for non master block {} using master state",
                self.id(),
            );
        }
        if block_id.seq_no() < block_info.prev_key_block_seqno() {
            bail!(
                "Can't check proof for block {} using master state {}, because it is older than the previous key block with seqno {}",
                self.id(),
                block_id,
                block_info.prev_key_block_seqno(),
            );
        }
        if block_id.seq_no() > self.id().seq_no() {
            bail!(
                "Can't check proof for block {} using newer master state {}",
                self.id(),
                block_id,
            );
        }

        let (cur_validator_set, cc_config) = state.read_cur_validator_set_and_cc_conf()?;

        let (validators, hash_short) = calc_subset_for_workchain(
            &cur_validator_set,
            config,
            &cc_config,
            self.id().shard().shard_prefix_with_tag(),
            self.id().shard().workchain_id(),
            self.signatures.as_ref().map(|s| s.validator_info.catchain_seqno).unwrap_or(0),
            block_info.gen_utime()
        )?;

        Ok((validators, hash_short))
    }
}

async fn get_current_network_zerostate_root_hash(
    context: &Arc<ClientContext>,
) -> ClientResult<Arc<String>> {
    if let Some(ref root_hash) = *context.net.zerostate_root_hash.read().await {
        return Ok(Arc::clone(root_hash));
    }

    let queried_root_hash = query_current_network_zerostate_root_hash(
        Arc::clone(context)
    ).await?;

    let mut write_guard = context.net.zerostate_root_hash.write().await;
    if let Some(ref stored_root_hash) = *write_guard {
        return Ok(Arc::clone(stored_root_hash));
    }

    *write_guard = Some(Arc::clone(&queried_root_hash));

    Ok(queried_root_hash)
}

async fn query_current_network_zerostate_root_hash(
    context: Arc<ClientContext>,
) -> ClientResult<Arc<String>> {
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
        result: "prev_ref{root_hash}".to_string(),
        limit: Some(1),
        ..Default::default()
    }).await?.result;

    if blocks.is_empty() {
        return Err(
            Error::unable_to_resolve_zerostate_root_hash("Can't get masterchain block #1")
        );
    }

    let prev_ref = &blocks[0]["prev_ref"];
    if prev_ref.is_null() {
        return Err(
            Error::unable_to_resolve_zerostate_root_hash("prev_ref of the block #1 is not set")
        );
    }

    let root_hash_json = &prev_ref["root_hash"];
    root_hash_json.as_str()
        .map(|v| Arc::new(v.to_string()))
        .ok_or_else::<ClientError, _>(|| Error::unable_to_resolve_zerostate_root_hash(
            format!(
                "root_hash of the prev_ref of the block #1 is not a string: {:?}",
                root_hash_json,
            ),
        ))
}

async fn resolve_initial_trusted_key_block(
    context: &Arc<ClientContext>,
) -> ClientResult<&TrustedMcBlockId> {
    let zerostate_root_hash = get_current_network_zerostate_root_hash(context).await?;

    if let Some(hardcoded_mc_block) =
        INITIAL_TRUSTED_KEY_BLOCKS.get(zerostate_root_hash.as_ref())
    {
        return Ok(&hardcoded_mc_block.trusted_key_block);
    }

    Err(Error::unable_to_resolve_trusted_key_block(&zerostate_root_hash))
}

#[async_trait::async_trait]
pub trait ProofHelperEngine {
    fn context(&self) -> &Arc<ClientContext>;
    async fn load_zerostate(&self) -> Result<ShardStateUnsplit>;
    async fn load_key_block_proof(&self, mc_seq_no: u32) -> Result<BlockProof>;
}
