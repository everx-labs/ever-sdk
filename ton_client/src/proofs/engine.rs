use std::convert::TryInto;
use std::future::Future;
use std::io::Cursor;
use std::ops::Range;
use std::str::FromStr;
use std::sync::Arc;

use failure::{bail, err_msg};
use serde_json::Value;
use ton_block::{BinTreeType, Block, Deserializable, InRefValue, ShardIdent, ShardStateUnsplit};
use ton_types::{deserialize_tree_of_cells, Result, UInt256};

use crate::boc::internal::get_boc_hash;
use crate::client::storage::{InMemoryKeyValueStorage, KeyValueStorage};
use crate::ClientContext;
use crate::encoding::base64_decode;
use crate::error::ClientResult;
use crate::net::{OrderBy, ParamsOfQueryCollection, query_collection, SortDirection};
use crate::proofs::{BlockProof, get_current_network_uid, ProofHelperEngine, resolve_initial_trusted_key_block};
use crate::proofs::Error;
use crate::utils::json::JsonHelper;

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

pub(crate) struct ProofHelperEngineImpl {
    context: Arc<ClientContext>,
    storage: Arc<dyn KeyValueStorage>,
}

impl ProofHelperEngineImpl {
    pub async fn new(context: Arc<ClientContext>) -> Result<Self> {
        let storage = Self::obtain_proof_storage(&context).await?;

        Ok(Self::with_values(context, storage))
    }

    pub fn with_values(context: Arc<ClientContext>, storage: Arc<dyn KeyValueStorage>) -> Self {
        Self { context, storage }
    }

    pub fn context(&self) -> &Arc<ClientContext> {
        &self.context
    }

    #[cfg(test)]
    pub fn storage(&self) -> &Arc<dyn KeyValueStorage> {
        &self.storage
    }

    async fn obtain_proof_storage(context: &Arc<ClientContext>) -> Result<Arc<dyn KeyValueStorage>> {
        if let Some(storage) = context.proofs_storage.read().await.as_ref() {
            return Ok(Arc::clone(storage));
        }

        let new_storage = if !context.config.proofs.cache_in_local_storage {
            Arc::new(InMemoryKeyValueStorage::new()) as Arc<dyn KeyValueStorage>
        } else {
            let network_uid = get_current_network_uid(&context).await?;

            let storage_name = format!(
                "proofs/{}/{}",
                Self::gen_root_hash_prefix(network_uid.zerostate_root_hash.as_slice()),
                Self::gen_root_hash_prefix(network_uid.first_master_block_root_hash.as_slice()),
            );
            Arc::new(
                crate::client::LocalStorage::new(
                    context.config.local_storage_path.clone(),
                    storage_name,
                ).await?
            ) as Arc<dyn KeyValueStorage>
        };

        let mut write_guard = context.proofs_storage.write().await;
        if let Some(storage) = write_guard.as_ref() {
            return Ok(Arc::clone(storage));
        }

        *write_guard = Some(Arc::clone(&new_storage));

        Ok(new_storage)
    }

    fn gen_root_hash_prefix(root_hash: &[u8]) -> String {
        hex::encode(&root_hash[..std::cmp::min(4, root_hash.len())])
    }

    fn mc_proof_key(mc_seq_no: u32) -> String {
        format!("proof_mc_{}", mc_seq_no)
    }

    fn block_key(root_hash: &str) -> String {
        format!("temp_block_{}", root_hash)
    }

    fn trusted_block_right_bound_key(seq_no: u32) -> String {
        format!("trusted_{}_right_boundary_seq_no", seq_no)
    }

    fn filter_for_block(root_hash: &str) -> Value {
        json!({
            "id": {
                "eq": root_hash,
            }
        })
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

    fn sorting_by_seq_no() -> Vec<OrderBy> {
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
            let seq_no = block.get_u32("seq_no")?;
            let gen_utime = block.get_u32("gen_utime")?;
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

    async fn get_value(&self, key: &str) -> Result<Option<Value>> {
        self.storage.get_str(key).await?
            .map(|value_str| serde_json::from_str(&value_str)
                .map_err(|err| err.into()))
            .transpose()
    }

    async fn put_value(&self, key: &str, value: &Value) -> Result<()> {
        self.storage.put_str(
            key,
            &serde_json::to_string(value)
                .map_err(|err| Error::internal_error(err))?,
        ).await
            .map_err(|err| err.into())
    }

    async fn read_mc_proof(&self, mc_seq_no: u32) -> Result<Option<Value>> {
        self.get_value(&Self::mc_proof_key(mc_seq_no)).await
    }

    async fn write_mc_block_proof(&self, mc_seq_no: u32, value: &Value) -> Result<()> {
        self.put_value(&Self::mc_proof_key(mc_seq_no), value).await
    }

    pub(crate) async fn read_block(&self, root_hash: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get_bin(&Self::block_key(root_hash)).await
            .map_err(|err| err.into())
    }

    pub(crate) async fn write_block(&self, root_hash: &str, boc: &[u8]) -> Result<()> {
        self.storage.put_bin(&Self::block_key(root_hash), boc).await
            .map_err(|err| err.into())
    }

    pub(crate) async fn read_metadata_value_u32(&self, key: &str) -> Result<Option<u32>> {
        Ok(
            self.storage.get_bin(key).await?
                .map(|vec|
                    vec.try_into()
                        .ok()
                        .map(|arr| u32::from_le_bytes(arr))
                ).flatten()
        )
    }

    pub(crate) async fn write_metadata_value_u32(&self, key: &str, value: u32) -> Result<()> {
        self.storage.put_bin(key, &value.to_le_bytes()).await
            .map_err(|err| err.into())
    }

    pub(crate) async fn update_metadata_value_u32(
        &self,
        key: &str,
        value: u32,
        process_value: fn(u32, u32) -> u32,
    ) -> Result<()> {
        match self.read_metadata_value_u32(key).await? {
            None => self.write_metadata_value_u32(key, value).await,
            Some(prev) => self.write_metadata_value_u32(key, process_value(prev, value)).await,
        }
    }

    pub(crate) async fn read_zs_right_bound(&self) -> Result<u32> {
        self.read_metadata_value_u32(ZEROSTATE_RIGHT_BOUND_KEY).await
            .map(|opt| opt.unwrap_or(0))
    }

    pub(crate) async fn update_zs_right_bound(&self, seq_no: u32) -> Result<()> {
        self.update_metadata_value_u32(ZEROSTATE_RIGHT_BOUND_KEY, seq_no, std::cmp::max).await
    }

    pub(crate) async fn read_trusted_block_right_bound(&self, trusted_seq_no: u32) -> Result<u32> {
        self.read_metadata_value_u32(&Self::trusted_block_right_bound_key(trusted_seq_no)).await
            .map(|opt| opt.unwrap_or(trusted_seq_no))
    }

    pub(crate) async fn update_trusted_block_right_bound(
        &self,
        trusted_seq_no: u32,
        right_bound_seq_no: u32,
    ) -> Result<()> {
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

        let boc = zerostates[0].get_str("boc")?;

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
                order: Some(Self::sorting_by_seq_no()),
                ..Default::default()
            }
        ).await?.result)?;

        if blocks.is_empty() {
            return Ok(None)
        }

        Ok(Some(blocks[0].1["prev_ref"].get_str("file_hash")?.to_string()))
    }

    pub(crate) async fn download_block_boc(
        &self,
        root_hash: &str,
    ) -> Result<Vec<u8>> {
        if let Some(boc) = self.read_block(root_hash).await? {
            Ok(boc)
        } else {
            let blocks = query_collection(
                Arc::clone(&self.context),
                ParamsOfQueryCollection {
                    collection: "blocks".to_string(),
                    result: "seq_no gen_utime boc".to_string(),
                    filter: Some(Self::filter_for_block(root_hash)),
                    limit: Some(1),
                    ..Default::default()
                }
            ).await?.result;

            if blocks.is_empty() {
                bail!(
                    "Unable to download block with `root_hash`: {} from DApp server",
                    root_hash,
                );
            }

            let block_json = &blocks[0];
            let boc_base64 = block_json.get_str("boc")?;

            Ok(base64::decode(boc_base64)?)
        }
    }

    pub(crate) async fn download_block_boc_and_calc_file_hash(
        &self,
        root_hash: &str,
    ) -> Result<UInt256> {
        let boc = self.download_block_boc(root_hash).await?;

        Ok(UInt256::calc_file_hash(&boc))
    }

    pub(crate) async fn query_mc_block_file_hash(
        &self,
        mc_seq_no: u32,
        root_hash: &str,
    ) -> Result<String> {
        if let Some(file_hash) = self.query_file_hash_from_next_block(mc_seq_no).await? {
            return Ok(file_hash);
        }
        let file_hash = self.download_block_boc_and_calc_file_hash(root_hash).await?;
        Ok(file_hash.as_hex_string())
    }

    pub(crate) async fn query_mc_block_proof(&self, mc_seq_no: u32) -> Result<Value> {
        let mut blocks = Self::preprocess_query_result(query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                result: PROOF_QUERY_RESULT.to_string(),
                filter: Some(Self::filter_for_mc_block(mc_seq_no)),
                order: Some(Self::sorting_by_seq_no()),
                ..Default::default()
            }
        ).await?.result)?;

        if blocks.is_empty() {
            bail!(
                "Unable to download proof for masterchain block with seq_no: {} from DApp server",
                mc_seq_no,
            );
        }

        let (seq_no, mut result) = blocks.remove(0);
        result["file_hash"] = self.query_mc_block_file_hash(
            seq_no,
            result.get_str("id")?,
        ).await?.into();

        Ok(result)
    }

    pub(crate) async fn check_mc_block_proof(
        &self,
        mc_seq_no: u32,
        root_hash: &UInt256,
    ) -> Result<()> {
        if let Some(proof_json) = self.read_mc_proof(mc_seq_no).await? {
            let id = UInt256::from_str(proof_json.get_str("id")?)?;
            if id != *root_hash {
                bail!(
                    "`id` ({}) of proven masterchain block with seq_no: {} mismatches `root_hash` \
                        ({}) of the block being checked",
                    id,
                    mc_seq_no,
                    root_hash,
                )
            }
            return Ok(());
        }

        let proof_json = self.query_mc_block_proof(mc_seq_no).await?;
        let proof = BlockProof::from_value(&proof_json)?;

        let expected_root_hash = proof.id().root_hash();

        if root_hash != expected_root_hash {
            bail!(
                "`root_hash` ({}) of downloaded proof for masterchain block with seq_no: {} \
                    mismatches `root_hash` ({}) of the block being checked",
                expected_root_hash,
                mc_seq_no,
                root_hash,
            )
        }

        proof.check_proof(self).await?;

        self.write_mc_block_proof(mc_seq_no, &proof_json).await?;

        Ok(())
    }

    pub(crate) async fn query_key_blocks_proofs(
        &self,
        mut mc_seq_no_range: Range<u32>,
    ) -> Result<Vec<(u32, Value)>> {
        let mut result = Vec::with_capacity(mc_seq_no_range.len());
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
                    order: Some(Self::sorting_by_seq_no()),
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

    pub(crate) async fn add_mc_blocks_file_hashes(
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
                    order: Some(Self::sorting_by_seq_no()),
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
        trusted_seq_no: u32,
        trusted_root_hash: &UInt256,
    ) -> Result<BlockProof> {
        let proof_json = self.query_mc_block_proof(trusted_seq_no).await?;
        let proof =  BlockProof::from_value(&proof_json)?;
        if proof.id().seq_no() != trusted_seq_no {
            bail!(
                "Proof for trusted key-block seq_no ({}) mismatches trusted key-block seq_no ({})",
                proof.id().seq_no,
                trusted_seq_no,
            );
        }
        if proof.id().root_hash() != trusted_root_hash {
            bail!(
                "Proof for trusted key-block root_hash ({:?}) mismatches trusted key-block root_hash ({:?})",
                proof.id().root_hash(),
                trusted_root_hash,
            )
        }
        self.write_mc_block_proof(trusted_seq_no, &proof_json).await?;

        Ok(proof)
    }

    async fn require_trusted_key_block_proof(
        &self,
        trusted_seq_no: u32,
        trusted_root_hash: &UInt256,
    ) -> Result<BlockProof> {
        if let Some(value) = self.read_mc_proof(trusted_seq_no).await? {
            return BlockProof::from_value(&value);
        }

        self.download_trusted_key_block_proof(trusted_seq_no, trusted_root_hash).await
    }

    pub(crate) async fn download_proof_chain<F: Fn(u32) -> R, R: Future<Output = Result<()>>>(
        &self,
        mc_seq_no_range: Range<u32>,
        on_store_block: F,
    ) -> Result<BlockProof> {
        if mc_seq_no_range.is_empty() {
            bail!("Empty masterchain seq_no range");
        }

        let mut proof_values = self.query_key_blocks_proofs(mc_seq_no_range).await?;
        self.add_mc_blocks_file_hashes(&mut proof_values).await?;

        let mut last_proof = None;
        for (mc_seq_no, proof_json) in proof_values {
            let proof = BlockProof::from_value(&proof_json)?;
            proof.check_proof(self).await?;

            self.write_mc_block_proof(mc_seq_no, &proof_json).await?;
            on_store_block(mc_seq_no).await?;

            last_proof = Some(proof);
        }

        last_proof.ok_or_else(|| err_msg("Empty proof chain"))
    }

    pub(crate) fn extract_top_shard_block(
        mc_block: &Block,
        shard: &ShardIdent,
    ) -> Result<(u32, UInt256)> {
        let extra = mc_block.read_extra()?;
        let mc_extra = extra.read_custom()?
            .ok_or_else(|| err_msg("Unable to read McBlockExtra"))?;

        let mut result = None;
        if let Some(InRefValue(bin_tree)) = mc_extra.shards().get(&shard.workchain_id())? {
            bin_tree.iterate(|prefix, shard_descr| {
                let shard_ident = ShardIdent::with_prefix_slice(shard.workchain_id(), prefix)?;
                if shard_ident != *shard {
                    return Ok(true);
                }
                result = Some((shard_descr.seq_no, shard_descr.root_hash));
                Ok(false)
            })?;
        }

        result.ok_or_else(
            || err_msg(format!("Top block for the given shard ({}) not found", shard))
        )
    }

    pub(crate) async fn query_closest_mc_block_for_shard_block(
        &self,
        first_mc_seq_no: &mut u32,
        shard: &ShardIdent,
        shard_block_seq_no: u32,
    ) -> Result<Option<u32>> {
        loop {
            let blocks = Self::preprocess_query_result(query_collection(
                Arc::clone(&self.context),
                ParamsOfQueryCollection {
                    collection: "blocks".to_string(),
                    result: "\
                        seq_no \
                        gen_utime \
                        master { \
                            shard_hashes { \
                                workchain_id \
                                shard \
                                descr { \
                                    seq_no \
                                    root_hash \
                                }\
                            }\
                        }".to_string(),
                    filter: Some(json!({
                        "workchain_id": { "eq": -1 },
                        "seq_no": { "ge": *first_mc_seq_no },
                    })),
                    order: Some(Self::sorting_by_seq_no()),
                    limit: Some(10),
                    ..Default::default()
                }
            ).await?.result)?;

            if blocks.is_empty() {
                return Ok(None);
            }

            for (seq_no, value) in &blocks {
                let shard_hashes = value["master"].get_array("shard_hashes")?;
                for item in shard_hashes {
                    if item["workchain_id"] == shard.workchain_id()
                        && item["shard"] == shard.shard_prefix_as_str_with_tag()
                        && item["descr"].get_u32("seq_no")? >= shard_block_seq_no
                    {
                        return Ok(Some(*seq_no))
                    }
                }
            }

            *first_mc_seq_no = blocks[blocks.len() - 1].0 + 1;
        }
    }

    pub(crate) async fn query_shard_block_bocs(
        &self,
        shard: &ShardIdent,
        seq_no_range: Range<u32>,
    ) -> Result<Vec<Vec<u8>>> {
        let blocks = Self::preprocess_query_result(query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                result: "\
                    seq_no \
                    gen_utime \
                    id \
                    boc \
                ".to_string(),
                filter: Some(json!({
                    "workchain_id": { "eq": shard.workchain_id() },
                    "shard": { "eq": shard.shard_prefix_as_str_with_tag() },
                    "seq_no": { "in": seq_no_range.clone().collect::<Vec<u32>>() },
                })),
                order: Some(Self::sorting_by_seq_no()),
                ..Default::default()
            }
        ).await?.result)?;

        if blocks.is_empty() {
            bail!(
                "No shard blocks found on DApp server for specified range \
                    (shard: {}, seq_no_range: {:?})",
                shard,
                seq_no_range,
            );
        }

        if blocks.len() != seq_no_range.len() {
            bail!(
                "Unexpected number of blocks returned by DApp server for specified range \
                    (shard: {}, seq_no_range: {:?}, expected count: {}, actual count: {})",
                shard,
                seq_no_range,
                seq_no_range.len(),
                blocks.len(),
            );
        }

        let mut result = Vec::with_capacity(blocks.len());
        for i in 0..blocks.len() {
            let (seq_no, block) = &blocks[i];

            let expected_seq_no = seq_no_range.start + i as u32;
            if *seq_no != expected_seq_no {
                bail!(
                    "Unexpected seq_no of block returned by DApp server for specified range \
                        (shard: {}, seq_no_range: {:?}, expected seq_no: {}, actual seq_no: {})",
                    shard,
                    seq_no_range,
                    expected_seq_no,
                    seq_no,
                );
            }

            result.push(base64_decode(block.get_str("boc")?)?);
        }

        Ok(result)
    }

    pub async fn check_shard_block(&self, boc: &[u8]) -> Result<()> {
        let cell = deserialize_tree_of_cells(&mut Cursor::new(boc))?;
        let root_hash = cell.repr_hash();
        let block = Block::construct_from_cell(cell)?;

        let info = block.info.read_struct()?;

        let master_ref = info.read_master_ref()?
            .ok_or_else(|| err_msg("Unable to read master_ref of block"))?;

        let mut first_mc_seq_no = master_ref.master.seq_no;
        loop {
            if let Some(mc_seq_no) = self.query_closest_mc_block_for_shard_block(
                    &mut first_mc_seq_no,
                    info.shard(),
                    info.seq_no(),
                ).await?
            {
                let mc_proof_json = self.query_mc_block_proof(mc_seq_no).await?;
                let mc_proof = BlockProof::from_value(&mc_proof_json)?;
                let (_mc_block, _mc_block_info) = mc_proof.check_proof(self).await?;

                let mc_boc = self.download_block_boc(
                    &mc_proof.id().root_hash().as_hex_string(),
                ).await?;
                let mc_cell = deserialize_tree_of_cells(&mut Cursor::new(&mc_boc))?;

                if mc_cell.repr_hash() != *mc_proof.id().root_hash() {
                    bail!(
                        "Proof checking failed: `root_hash` of MC block's BOC downloaded from DApp \
                            server mismatches `root_hash` of proof for this MC block",
                    );
                }

                self.write_block(&mc_cell.repr_hash().as_hex_string(), &mc_boc).await?;

                let mc_block = Block::construct_from_bytes(&mc_boc)?;

                let (top_seq_no, top_root_hash) =
                    Self::extract_top_shard_block(&mc_block, info.shard())?;

                if top_seq_no == info.seq_no() {
                    if top_root_hash != root_hash {
                        bail!("Proof checking failed: masterchain block references shard block \
                            with different `root_hash`: reference {}, but shard block has {}",
                            top_root_hash,
                            root_hash,
                        );
                    }
                    return Ok(());
                }

                let shard_chain = self.query_shard_block_bocs(
                    info.shard(),
                    (info.seq_no() + 1)..(top_seq_no + 1),
                ).await?;


                let check_with_last_prev_ref =
                    |seq_no, root_hash, last_prev_ref_seq_no, last_prev_ref_root_hash| {
                        if seq_no != last_prev_ref_seq_no {
                            bail!(
                                "Queried shard block's `seq_no` ({}) mismatches `prev_ref.seq_no` ({}) \
                                    of the next block or reference from the masterchain block",
                                seq_no,
                                last_prev_ref_seq_no,
                            );
                        }

                        if root_hash != last_prev_ref_root_hash {
                            bail!(
                                "Shard block proof checking failed: \
                                    block's `root_hash` ({}) mismatches `prev_ref.root_hash` ({}) \
                                    of the next block or reference from the masterchain block",
                                root_hash,
                                last_prev_ref_root_hash,
                            );
                        }

                        Ok(())
                    };

                let mut last_prev_ref_seq_no = top_seq_no;
                let mut last_prev_ref_root_hash = top_root_hash;
                for boc in shard_chain.iter().rev() {
                    let cell = deserialize_tree_of_cells(&mut Cursor::new(boc))?;
                    let root_hash = cell.repr_hash();
                    let block = Block::construct_from_cell(cell)?;
                    let info = block.read_info()?;

                    check_with_last_prev_ref(
                        info.seq_no(),
                        root_hash,
                        last_prev_ref_seq_no,
                        last_prev_ref_root_hash,
                    )?;

                    let prev_ref = info.read_prev_ref()?.prev1()?;
                    last_prev_ref_seq_no = prev_ref.seq_no;
                    last_prev_ref_root_hash = prev_ref.root_hash;
                }

                check_with_last_prev_ref(
                    info.seq_no(),
                    root_hash,
                    last_prev_ref_seq_no,
                    last_prev_ref_root_hash,
                )?;

                return Ok(());
            }

            // TODO: Rewrite using wait_for():
            self.context.env.set_timer(1000).await.ok();
        }
    }

    pub(crate) async fn query_transaction_data(&self, id: &str, fields: &str) -> Result<Value> {
        let mut transactions = query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "transactions".to_string(),
                result: fields.to_string(),
                filter: Some(json!({
                    "id": {
                        "eq": id,
                    },
                })),
                limit: Some(1),
                ..Default::default()
            }
        ).await?.result;

        if transactions.is_empty() {
            bail!("Unable to download transaction data from DApp server");
        }

        Ok(transactions.remove(0))
    }

    pub(crate) async fn query_message_data(&self, id: &str, fields: &str) -> Result<Value> {
        let mut messages = query_collection(
            Arc::clone(&self.context),
            ParamsOfQueryCollection {
                collection: "messages".to_string(),
                result: fields.to_string(),
                filter: Some(json!({
                    "id": {
                        "eq": id,
                    },
                })),
                limit: Some(1),
                ..Default::default()
            }
        ).await?.result;

        if messages.is_empty() {
            bail!("Unable to download message data from DApp server");
        }

        Ok(messages.remove(0))
    }

    pub(crate) async fn proof_block_boc(
        &self,
        root_hash: &UInt256,
        block: &Block,
        boc: &Vec<u8>,
    ) -> ClientResult<()> {
        // TODO: Manage untrusted and trusted (already proven) blocks separately.
        //       For trusted blocks we don't need to do proof checking.
        //       1. `write_block()` change to `write_untrusted_block()`
        //       2. also add `write_trusted_block()` and `remove_untrusted_block()`
        //          (or `trust_block()` for moving block from untrusted to trusted storage) functions.
        self.write_block(&root_hash.as_hex_string(), &boc).await
            .map_err(|err| Error::internal_error(err))?;

        let info = block.read_info()
            .map_err(|err| Error::invalid_data(err))?;
        if info.shard().is_masterchain() {
            self.check_mc_block_proof(info.seq_no(), &root_hash).await
        } else {
            self.check_shard_block(&boc).await
        }.map_err(|err| Error::proof_check_failed(err))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProofHelperEngine for ProofHelperEngineImpl {
    async fn load_zerostate(&self) -> Result<ShardStateUnsplit> {
        if let Some(boc) = self.storage.get_bin(ZEROSTATE_KEY).await? {
            return ShardStateUnsplit::construct_from_bytes(&boc);
        }

        let boc = self.query_zerostate_boc().await?;

        let actual_hash = UInt256::from_str(&get_boc_hash(&boc)?)?;
        let network_uid = get_current_network_uid(self.context()).await?;
        if actual_hash != network_uid.zerostate_root_hash {
            bail!(
                "Zerostate hashes mismatch (expected `{:x}`, but queried from DApp is `{:x}`)",
                network_uid.zerostate_root_hash,
                actual_hash,
            );
        }

        self.storage.put_bin(ZEROSTATE_KEY, &boc).await?;

        ShardStateUnsplit::construct_from_bytes(&boc)
    }

    async fn load_key_block_proof(&self, mc_seq_no: u32) -> Result<BlockProof> {
        if let Some(proof_json) = self.read_mc_proof(mc_seq_no).await? {
            return BlockProof::from_value(&proof_json);
        }

        let (trusted_seq_no, trusted_root_hash) = resolve_initial_trusted_key_block(self.context(), mc_seq_no).await?;
        let zs_right_bound = self.read_zs_right_bound().await?;
        let trusted_right_bound = self.read_trusted_block_right_bound(trusted_seq_no).await?;

        if mc_seq_no == trusted_seq_no {
            return self.download_trusted_key_block_proof(trusted_seq_no, &trusted_root_hash).await;
        }

        self.require_trusted_key_block_proof(trusted_seq_no, &trusted_root_hash).await?;

        let update_zs_right = move |mc_seq_no| async move {
            self.update_zs_right_bound(mc_seq_no).await
        };

        let update_trusted_right = move |mc_seq_no| async move {
            self.update_trusted_block_right_bound(trusted_seq_no, mc_seq_no).await
        };

        if mc_seq_no > trusted_right_bound {
            self.download_proof_chain(trusted_right_bound + 1..mc_seq_no + 1, update_trusted_right).await
        } else if mc_seq_no < trusted_seq_no && mc_seq_no > zs_right_bound {
            self.download_proof_chain(zs_right_bound + 1..mc_seq_no + 1, update_zs_right).await
        } else if mc_seq_no <= zs_right_bound {
            // Chain from zerostate is broken
            self.download_proof_chain(1..mc_seq_no + 1, update_zs_right).await
        } else if mc_seq_no > trusted_seq_no && mc_seq_no <= trusted_right_bound {
            // Chain from trusted key-block to the right is broken
            self.download_proof_chain(trusted_seq_no + 1..mc_seq_no + 1, update_trusted_right).await
        } else {
            unreachable!(
                "mc_seq_no: {}, zs_right: {}, trusted_right: {}, trusted_seq_no: {:?}",
                mc_seq_no,
                zs_right_bound,
                trusted_right_bound,
                trusted_seq_no,
            )
        }
    }
}
