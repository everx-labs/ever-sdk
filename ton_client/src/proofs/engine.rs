use std::convert::TryInto;
use std::future::Future;
use std::ops::Range;
use std::sync::Arc;

use failure::{bail, err_msg};
use serde_json::Value;
use ton_block::{Deserializable, ShardStateUnsplit};
use ton_types::{Result, UInt256};

use crate::boc::internal::get_boc_hash;
use crate::client::{Error, NetworkUID};
use crate::ClientContext;
use crate::error::ClientResult;
use crate::net::{OrderBy, ParamsOfQueryCollection, query_collection, SortDirection};
use crate::net::types::TrustedMcBlockId;
use crate::proofs::{BlockProof, get_current_network_uid, ProofHelperEngine, resolve_initial_trusted_key_block, storage::ProofStorage};

const ZEROSTATE_KEY: &str = "zerostate";
const ZEROSTATE_RIGHT_BOUND_KEY: &str = "zs_right_boundary_seq_no";
const PROOF_QUERY_RESULT: &str = "\
    id \
    workchain_id \
    shard \
    seq_no \
    gen_utime \
    signatures {\
        proof \
        catchain_seqno \
        validator_list_hash_short \
        sig_weight \
        signatures {\
            node_id \
            r \
            s\
        }\
    }\
";

pub struct ProofHelperEngineImpl<Storage: ProofStorage + Send + Sync> {
    context: Arc<ClientContext>,
    storage: Arc<Storage>,
}

impl<Storage: ProofStorage + Send + Sync> ProofHelperEngineImpl<Storage> {
    pub fn new(context: Arc<ClientContext>, storage: Arc<Storage>) -> Self {
        Self { context, storage }
    }

    pub fn context(&self) -> &Arc<ClientContext> {
        &self.context
    }

    pub fn storage(&self) -> &Arc<Storage> {
        &self.storage
    }

    fn gen_root_hash_prefix(root_hash: &str) -> &str {
        &root_hash[..std::cmp::min(8, root_hash.len())]
    }

    pub(crate) fn gen_storage_key(network_uid: &NetworkUID, key: &str) -> String {
        format!(
            "{}/{}/{}",
            Self::gen_root_hash_prefix(&network_uid.zerostate_root_hash),
            Self::gen_root_hash_prefix(&network_uid.first_master_block_root_hash),
            key,
        )
    }

    pub(crate) async fn get_storage_key(&self, key: &str) -> ClientResult<String> {
        let network_uid = get_current_network_uid(&self.context).await?;
        Ok(Self::gen_storage_key(&network_uid, key))
    }

    fn mc_proof_key(mc_seq_no: u32) -> String {
        format!("proof_mc_{}", mc_seq_no)
    }

    fn mc_block_key(mc_seq_no: u32) -> String {
        format!("block_mc_{}", mc_seq_no)
    }

    fn trusted_block_right_bound_key(seq_no: u32) -> String {
        format!("trusted_{}_right_boundary_seq_no", seq_no)
    }

    fn filter_for_mc_block(mc_seq_no: u32) -> Value {
        json!({
            "workchain_id": {
                "eq": -1,
            },
            "seq_no": {
                "eq": mc_seq_no,
            }
        })
    }

    fn sorting_for_mc() -> Vec<OrderBy> {
        vec![
            OrderBy {
                path: "seq_no".to_string(),
                direction: SortDirection::ASC,
            },
        ]
    }

    fn preprocess_query_result(blocks: Vec<Value>) -> Result<Vec<(u32, Value)>> {
        let mut result = Vec::with_capacity(blocks.len());

        let mut last_seq_no = 0;
        let mut last_gen_utime = 0;
        for block in blocks {
            let seq_no = block["seq_no"].as_u64()
                .ok_or_else(|| err_msg("seq_no of block must be an integer value"))? as u32;
            let gen_utime = block["gen_utime"].as_u64()
                .ok_or_else(|| err_msg("gen_utime of block must be an integer value"))? as u32;
            if seq_no != last_seq_no {
                result.push((seq_no, block));
                last_seq_no = seq_no;
                last_gen_utime =  gen_utime;
            } else if gen_utime > last_gen_utime {
                let last_index = result.len() - 1;
                result[last_index].1 = block;
                last_gen_utime = gen_utime;
            }
        }

        Ok(result)
    }

    async fn get_bin(&self, key: &str) -> ClientResult<Option<Vec<u8>>> {
        self.storage.get_bin(&self.get_storage_key(key).await?).await
    }

    async fn put_bin(&self, key: &str, value: &[u8]) -> ClientResult<()> {
        self.storage.put_bin(&self.get_storage_key(key).await?, value).await
    }

    async fn get_str(&self, key: &str) -> ClientResult<Option<String>> {
        self.storage.get_str(&self.get_storage_key(key).await?).await
    }

    async fn put_str(&self, key: &str, value: &str) -> ClientResult<()> {
        self.storage.put_str(&self.get_storage_key(key).await?, value).await
    }

    async fn get_value(&self, key: &str) -> ClientResult<Option<Value>> {
        self.get_str(key).await?
            .map(|value_str| serde_json::from_str(&value_str)
                .map_err(|err| Error::internal_error(err)))
            .transpose()
    }

    async fn put_value(&self, key: &str, value: &Value) -> ClientResult<()> {
        self.put_str(
            key,
            &serde_json::to_string(value)
                .map_err(|err| Error::internal_error(err))?,
        ).await
    }

    async fn read_mc_proof(&self, mc_seq_no: u32) -> ClientResult<Option<Value>> {
        self.get_value(&Self::mc_proof_key(mc_seq_no)).await
    }

    async fn write_mc_proof(&self, mc_seq_no: u32, value: &Value) -> ClientResult<()> {
        self.put_value(&Self::mc_proof_key(mc_seq_no), value).await
    }

    async fn read_mc_block(&self, mc_seq_no: u32) -> ClientResult<Option<Vec<u8>>> {
        self.get_bin(&Self::mc_block_key(mc_seq_no)).await
    }

    async fn write_mc_block(&self, mc_seq_no: u32, boc: &[u8]) -> ClientResult<()> {
        self.put_bin(&Self::mc_block_key(mc_seq_no), boc).await
    }

    pub(crate) async fn read_metadata_value_u32(&self, key: &str) -> ClientResult<Option<u32>> {
        Ok(
            self.get_bin(key).await?
                .map(|vec|
                    vec.try_into()
                        .ok()
                        .map(|arr| u32::from_le_bytes(arr))
                ).flatten()
        )
    }

    pub(crate) async fn write_metadata_value_u32(&self, key: &str, value: u32) -> ClientResult<()> {
        self.put_bin(key, &value.to_le_bytes()).await
    }

    pub(crate) async fn update_metadata_value_u32(
        &self,
        key: &str,
        value: u32,
        process_value: fn(u32, u32) -> u32,
    ) -> ClientResult<()> {
        match self.read_metadata_value_u32(key).await? {
            None => self.write_metadata_value_u32(key, value).await,
            Some(prev) => self.write_metadata_value_u32(key, process_value(prev, value)).await,
        }
    }

    pub(crate) async fn read_zs_right_bound(&self) -> ClientResult<u32> {
        self.read_metadata_value_u32(ZEROSTATE_RIGHT_BOUND_KEY).await
            .map(|opt| opt.unwrap_or(0))
    }

    pub(crate) async fn update_zs_right_bound(&self, seq_no: u32) -> ClientResult<()> {
        self.update_metadata_value_u32(ZEROSTATE_RIGHT_BOUND_KEY, seq_no, std::cmp::max).await
    }

    pub(crate) async fn read_trusted_block_right_bound(&self, trusted_seq_no: u32) -> ClientResult<u32> {
        self.read_metadata_value_u32(&Self::trusted_block_right_bound_key(trusted_seq_no)).await
            .map(|opt| opt.unwrap_or(trusted_seq_no))
    }

    pub(crate) async fn update_trusted_block_right_bound(
        &self,
        trusted_seq_no: u32,
        right_bound_seq_no: u32,
    ) -> ClientResult<()> {
        self.update_metadata_value_u32(
            &Self::trusted_block_right_bound_key(trusted_seq_no),
            right_bound_seq_no,
            std::cmp::max,
        ).await
    }

    pub(crate) async fn query_zerostate_boc(&self) -> Result<Vec<u8>> {
        let zerostates = query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "zerostates".to_string(),
                result: "boc".to_string(),
                limit: Some(1),
                ..Default::default()
            }
        ).await?.result;

        if zerostates.is_empty() {
            bail!("Unable to download network's zerostate from DApp server");
        }

        let boc = zerostates[0]["boc"].as_str()
            .ok_or_else(|| err_msg("BoC of zerostate must be a string"))?;

        Ok(base64::decode(boc)?)
    }

    pub(crate) async fn query_file_hash_from_next_block(
        &self,
        mut mc_seq_no: u32,
    ) -> Result<Option<String>> {
        mc_seq_no += 1;
        let blocks = Self::preprocess_query_result(query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                result: "seq_no gen_utime prev_ref{file_hash}".to_string(),
                filter: Some(Self::filter_for_mc_block(mc_seq_no)),
                order: Some(Self::sorting_for_mc()),
                ..Default::default()
            }
        ).await?.result)?;

        if blocks.is_empty() {
            return Ok(None)
        }

        Ok(Some(blocks[0].1["prev_ref"]["file_hash"].as_str()
            .ok_or_else(|| err_msg("file_hash field must be a string"))?
            .to_string()))
    }

    pub(crate) async fn download_boc_and_calc_file_hash(
        &self,
        mc_seq_no: u32,
    ) -> Result<UInt256> {
        let boc = if let Some(boc) = self.read_mc_block(mc_seq_no).await? {
            boc
        } else {
            let blocks = Self::preprocess_query_result(query_collection(
                Arc::clone(&self.context),
                ParamsOfQueryCollection {
                    collection: "blocks".to_string(),
                    result: "seq_no gen_utime boc".to_string(),
                    filter: Some(Self::filter_for_mc_block(mc_seq_no)),
                    order: Some(Self::sorting_for_mc()),
                    ..Default::default()
                }
            ).await?.result)?;

            if blocks.is_empty() {
                bail!(
                    "Unable to download masterchain block with seq_no: {} from DApp server",
                    mc_seq_no,
                );
            }

            let (_seq_no, block_json) = &blocks[0];
            let boc_base64 = block_json["boc"].as_str()
                .ok_or_else(|| err_msg("boc field must be a string"))?;
            let boc = base64::decode(boc_base64)?;

            self.write_mc_block(mc_seq_no, &boc).await?;

            boc
        };

        Ok(UInt256::calc_file_hash(&boc))
    }

    pub(crate) async fn query_mc_proof(&self, mc_seq_no: u32) -> Result<Value> {
        let mut blocks = Self::preprocess_query_result(query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                result: PROOF_QUERY_RESULT.to_string(),
                filter: Some(Self::filter_for_mc_block(mc_seq_no)),
                order: Some(Self::sorting_for_mc()),
                ..Default::default()
            }
        ).await?.result)?;

        if blocks.is_empty() {
            bail!(
                "Unable to download proof for masterchain block with seq_no: {} from DApp server",
                mc_seq_no,
            );
        }

        let (_seq_no, mut result) = blocks.remove(0);
        if let Some(file_hash) = self.query_file_hash_from_next_block(mc_seq_no).await? {
            result["file_hash"] = file_hash.into();
        } else {
            let file_hash = self.download_boc_and_calc_file_hash(mc_seq_no).await?;
            result["file_hash"] = file_hash.as_hex_string().into();
        }

        Ok(result)
    }

    pub(crate) async fn query_key_blocks_proofs(
        &self,
        mut mc_seq_no_range: Range<u32>,
    ) -> Result<Vec<(u32, Value)>> {
        let mut result = Vec::new();
        loop {
            if mc_seq_no_range.is_empty() {
                return Ok(result);
            }

            let key_blocks = query_collection(
                Arc::clone(&self.context),
                ParamsOfQueryCollection {
                    collection: "blocks".to_string(),
                    result: PROOF_QUERY_RESULT.to_string(),
                    filter: Some(json!({
                        "workchain_id": {
                            "eq": -1,
                        },
                        "key_block": {
                            "eq": true,
                        },
                        "seq_no": {
                            "ge": mc_seq_no_range.start,
                            "lt": mc_seq_no_range.end,
                        }
                    })),
                    order: Some(Self::sorting_for_mc()),
                    ..Default::default()
                }
            ).await?.result;

            if key_blocks.is_empty() {
                return Ok(result);
            }

            result.append(&mut Self::preprocess_query_result(key_blocks)?);
            mc_seq_no_range.start = result[result.len() - 1].0 + 1;
        }
    }

    pub(crate) async fn add_file_hashes(
        &self,
        mut proofs_sorted: &mut [(u32, Value)],
    ) -> Result<()> {
        while proofs_sorted.len() > 0 {
            let mut blocks = Self::preprocess_query_result(query_collection(
                Arc::clone(&self.context),
                ParamsOfQueryCollection {
                    collection: "blocks".to_string(),
                    result: "seq_no gen_utime prev_ref{file_hash}".to_string(),
                    filter: Some(json!({
                        "workchain_id": {
                            "eq": -1,
                        },
                        "seq_no": {
                            "in": proofs_sorted.iter()
                                    .map(|(seq_no, _value)| *seq_no + 1)
                                    .collect::<Vec<u32>>(),
                        }
                    })),
                    order: Some(Self::sorting_for_mc()),
                    ..Default::default()
                }
            ).await?.result)?;

            if proofs_sorted.len() < blocks.len() {
                bail!(
                    "DApp server returned more blocks ({}) than expected ({})",
                    blocks.len(),
                    proofs_sorted.len(),
                )
            }

            let (expected, remaining) = proofs_sorted.split_at_mut(blocks.len());
            for i in 0..blocks.len() {
                let (seq_no, mut block) = blocks.remove(0);

                let expected_seq_no = expected[i].0 + 1;
                if seq_no != expected_seq_no {
                    bail!(
                        "Block with seq_no: {} missed on DApp server (actual seq_no: {})",
                        expected_seq_no,
                        seq_no,
                    );
                }

                expected[i].1["file_hash"] = block["prev_ref"]["file_hash"].take();
            }

            proofs_sorted = remaining;
        }

        Ok(())
    }

    pub(crate) async fn download_trusted_key_block_proof(
        &self,
        id: &TrustedMcBlockId,
    ) -> Result<BlockProof> {
        let proof_json = self.query_mc_proof(id.seq_no).await?;
        let proof =  BlockProof::from_value(&proof_json)?;
        if proof.id().seq_no() != id.seq_no {
            bail!(
                "Proof for trusted key-block seq_no ({}) mismatches trusted key-block seq_no ({})",
                proof.id().seq_no,
                id.seq_no,
            );
        }
        let trusted_root_hash = UInt256::from_str(&id.root_hash)?;
        if proof.id().root_hash() != trusted_root_hash {
            bail!(
                "Proof for trusted key-block root_hash ({:?}) mismatches trusted key-block root_hash ({:?})",
                proof.id().root_hash(),
                trusted_root_hash,
            )
        }
        self.write_mc_proof(id.seq_no, &proof_json).await?;

        Ok(proof)
    }

    async fn require_trusted_key_block_proof(
        &self,
        id: &TrustedMcBlockId,
    ) -> Result<BlockProof> {
        if let Some(value) = self.read_mc_proof(id.seq_no).await? {
            return BlockProof::from_value(&value);
        }

        self.download_trusted_key_block_proof(id).await
    }

    pub(crate) async fn download_proof_chain<F: Fn(u32) -> R, R: Future<Output = ClientResult<()>>>(
        &self,
        mc_seq_no_range: Range<u32>,
        on_store_block: F,
    ) -> Result<BlockProof> {
        if mc_seq_no_range.is_empty() {
            bail!("Empty materchain seq_no range");
        }

        let mut proof_values = self.query_key_blocks_proofs(mc_seq_no_range).await?;
        self.add_file_hashes(&mut proof_values).await?;

        let mut last_proof = None;
        for (mc_seq_no, proof_json) in proof_values {
            let proof = BlockProof::from_value(&proof_json)?;
            proof.check_proof(self).await?;

            self.write_mc_proof(mc_seq_no, &proof_json).await?;
            on_store_block(mc_seq_no).await?;

            last_proof = Some(proof);
        }

        last_proof.ok_or_else(|| err_msg("Empty proof chain"))
    }
}

#[async_trait::async_trait]
impl<Storage: ProofStorage + Send + Sync> ProofHelperEngine for ProofHelperEngineImpl<Storage> {
    async fn load_zerostate(&self) -> Result<ShardStateUnsplit> {
        if let Some(boc) = self.get_bin(ZEROSTATE_KEY).await? {
            return ShardStateUnsplit::construct_from_bytes(&boc);
        }

        let boc = self.query_zerostate_boc().await?;

        let actual_hash = get_boc_hash(&boc)?;
        let expected_hash = &get_current_network_uid(self.context()).await?
            .zerostate_root_hash;
        if actual_hash != *expected_hash {
            bail!(
                "Zerostate hashes mismatch (expected `{}`, but queried from DApp is `{}`)",
                expected_hash,
                actual_hash,
            );
        }

        self.put_bin(ZEROSTATE_KEY, &boc).await?;

        ShardStateUnsplit::construct_from_bytes(&boc)
    }

    async fn load_key_block_proof(&self, mc_seq_no: u32) -> Result<BlockProof> {
        if let Some(proof_json) = self.read_mc_proof(mc_seq_no).await? {
            return BlockProof::from_value(&proof_json);
        }

        let trusted_id = resolve_initial_trusted_key_block(self.context()).await?;
        let zs_right_bound = self.read_zs_right_bound().await?;
        let trusted_right_bound = self.read_trusted_block_right_bound(trusted_id.seq_no).await?;

        if mc_seq_no == trusted_id.seq_no {
            return self.download_trusted_key_block_proof(trusted_id).await;
        }

        self.require_trusted_key_block_proof(trusted_id).await?;

        let update_zs_right = move |mc_seq_no| async move {
            self.update_zs_right_bound(mc_seq_no).await
        };

        let update_trusted_right = move |mc_seq_no| async move {
            self.update_trusted_block_right_bound(trusted_id.seq_no, mc_seq_no).await
        };

        if mc_seq_no > trusted_right_bound {
            self.download_proof_chain(trusted_right_bound + 1..mc_seq_no + 1, update_trusted_right).await
        } else if mc_seq_no < trusted_id.seq_no && mc_seq_no > zs_right_bound {
            self.download_proof_chain(zs_right_bound + 1..mc_seq_no + 1, update_zs_right).await
        } else if mc_seq_no <= zs_right_bound {
            // Chain from zerostate is broken
            self.download_proof_chain(1..mc_seq_no + 1, update_zs_right).await
        } else if mc_seq_no > trusted_id.seq_no && mc_seq_no <= trusted_right_bound {
            // Chain from trusted key-block to the right is broken
            self.download_proof_chain(trusted_id.seq_no + 1..mc_seq_no + 1, update_trusted_right).await
        } else {
            unreachable!(
                "mc_seq_no: {}, zs_right: {}, trusted_right: {}, trusted_id: {:?}",
                mc_seq_no,
                zs_right_bound,
                trusted_right_bound,
                trusted_id,
            )
        }
    }
}
