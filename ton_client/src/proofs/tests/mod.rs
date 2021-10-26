use std::sync::Arc;

use serde_json::Value;
use ton_block::{BinTreeType, Block, BlockIdExt, Deserializable, InRefValue, MASTERCHAIN_ID, ShardHashes, ShardIdent, ShardStateUnsplit};
use ton_types::{Result, UInt256};

use crate::client::storage::InMemoryKeyValueStorage;
use crate::proofs::{BlockProof, get_current_network_uid, INITIAL_TRUSTED_KEY_BLOCKS, query_current_network_uid, resolve_initial_trusted_key_block, ParamsOfProofBlockData};
use crate::proofs::engine::ProofHelperEngineImpl;
use crate::proofs::validators::{calc_subset_for_workchain, calc_workchain_id, calc_workchain_id_by_adnl_id};
use crate::tests::TestClient;
use crate::net::{ParamsOfQueryCollection, query_collection};
use crate::ClientContext;

#[test]
fn test_check_master_blocks_proof() -> Result<()> {
    let key_block_proof = BlockProof::read_from_file(
        "src/proofs/tests/data/test_master_block_proof/key_proof__3082181"
    )?;

    for seq_no in 3082182..=3082200 {
        let block_proof = BlockProof::read_from_file(
            format!("src/proofs/tests/data/test_master_block_proof/proof__{}", seq_no)
        )?;
        let (virt_block, virt_block_info) = block_proof.pre_check_block_proof()?;
        block_proof.check_with_prev_key_block_proof(&key_block_proof, &virt_block, &virt_block_info)?;
    }

    Ok(())
}

#[test]
fn test_check_master_blocks_proof_shuffle() -> Result<()> {
    let key_block_proof = BlockProof::read_from_file(
        "src/proofs/tests/data/test_master_block_proof_shuffle/key_proof__3236530"
    )?;

    for seq_no in 3236531..=3236550 {
        let block_proof = BlockProof::read_from_file(
            format!("src/proofs/tests/data/test_master_block_proof_shuffle/proof__{}", seq_no)
        )?;

        let (virt_block, virt_block_info) = block_proof.pre_check_block_proof()?;
        block_proof.check_with_prev_key_block_proof(&key_block_proof, &virt_block, &virt_block_info)?;
    }

    Ok(())
}

#[test]
fn test_calc_workchain_id_by_adnl_id() {
    assert_eq!(calc_workchain_id_by_adnl_id(&[0; 32]), -1);
    assert_eq!(calc_workchain_id_by_adnl_id(&[1; 32]), 0);
    assert_eq!(calc_workchain_id_by_adnl_id(&[2; 32]), 1);
    assert_eq!(calc_workchain_id_by_adnl_id(&[3; 32]), 2);
}

#[test]
fn test_validator_set() -> Result<()> {
    let block = Block::construct_from_file("src/proofs/tests/data/key_block.boc")?;
    let custom = block.read_extra()?.read_custom()?.unwrap();
    let config = custom.config().unwrap();
    assert!(config.prev_validator_set_present()?, "key block must be after elections");

    let vset = config.validator_set()?;
    assert_eq!(vset.list().len(), 21);

    let election_id = vset.utime_since();
    assert_eq!(election_id, 1627898896);

    let cc_seqno = block.read_info()?.gen_catchain_seqno();

    // vset.list().iter().enumerate().for_each(|(i,descr)| {
    //     let real_id = calc_workchain_id(descr);
    //     println!("{}: pub_key: {} real_id: {}", i, hex::encode(descr.public_key.as_slice()), real_id);
    // });

    for workchain_id in -1..=1 {
        // println!("workchain_id: {}", workchain_id);
        let cc_config = config.catchain_config()?;
        let subset = calc_subset_for_workchain(&vset, config, &cc_config, ton_block::SHARD_FULL, workchain_id, cc_seqno, 0.into())?;
        assert_eq!(subset.0.len(), 7);
        subset.0.iter().enumerate().for_each(|(_i,descr)| {
            let real_id = calc_workchain_id(descr);
            // println!("{}: pub_key: {} real_id: {}", i, hex::encode(descr.public_key.as_slice()), real_id);
            assert_eq!(real_id, workchain_id);
        });
    }

    Ok(())
}

#[test]
fn test_any_keyblock_validator_set() -> Result<()> {
    check_any_keyblock_validator_set("src/proofs/tests/data/key_block.boc")
}

fn get_top_blocks(shards: &ShardHashes, workchains: &[i32]) -> Result<Vec<BlockIdExt>> {
    let mut result = Vec::new();
    for workchain_id in workchains {
        if let Some(InRefValue(bintree)) = shards.get(workchain_id)? {
            bintree.iterate(|prefix, shard_descr| {
                let shard_ident = ShardIdent::with_prefix_slice(*workchain_id, prefix)?;
                let block_id = BlockIdExt::with_params(shard_ident, shard_descr.seq_no, shard_descr.root_hash, shard_descr.file_hash);
                result.push(block_id);
                Ok(true)
            })?;
        }
    }
    Ok(result)
}

fn check_any_keyblock_validator_set(file_name: &str) -> Result<()> {
    let block = Block::construct_from_file(file_name)?;
    let custom = block.read_extra()?.read_custom()?.unwrap();
    let config = custom.config().unwrap();

    // let vset = config.validator_set()?;
    // let election_id = vset.utime_since();
    // println!("elections: {} total validators: {}", election_id, vset.list().len());

    let cc_seqno = block.read_info()?.gen_catchain_seqno();

    // vset.list().iter().enumerate().for_each(|(i,descr)| {
    //     let id = calc_workchain_id(descr);
    //     println!("{}: pub_key: {} id: {}", i, hex::encode(descr.public_key.as_slice()), id);
    // });

    let count = config.workchains()?.len()? as i32;
    for workchain_id in -1..count {
        let shard_ids = match workchain_id {
            MASTERCHAIN_ID => vec!(BlockIdExt::with_params(ShardIdent::masterchain(), 0, Default::default(), Default::default())),
            workchain_id => get_top_blocks(custom.shards(), &[workchain_id])?
        };
        for block_id in shard_ids {
            // println!("{}", block_id.shard());
            let vset = config.validator_set()?;
            let cc_config = config.catchain_config()?;
            let subset = calc_subset_for_workchain(
                &vset,
                config,
                &cc_config,
                block_id.shard().shard_prefix_with_tag(),
                workchain_id,
                cc_seqno,
                Default::default()
            )?;
            assert_eq!(subset.0.len(), 7);
            subset.0.iter().enumerate().for_each(|(_i, descr)| {
                let real_id = calc_workchain_id(descr);
                // println!("{}: pub_key: {} real_id: {}", i, hex::encode(descr.public_key.as_slice()), real_id);
                assert_eq!(real_id, workchain_id);
            });
        }
    }

    Ok(())
}

const MAINNET_ZEROSTATE_ROOT_HASH: &str =
    "58ffca1a178daff705de54216e5433c9bd2e7d850070d334d38997847ab9e845";

lazy_static! {
    static ref MAINNET_CONFIG: Value = json!({
        "network": {
            "server_address": "main.ton.dev",
        }
    });
}

#[tokio::test]
async fn test_query_current_network_zerostate_root_hash() -> Result<()> {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());

    assert_eq!(
        query_current_network_uid(client.context()).await?.zerostate_root_hash.as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    Ok(())
}

#[tokio::test]
async fn test_get_current_network_zerostate_root_hash() -> Result<()> {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());
    let context = client.context();

    assert!(context.net.network_uid.read().await.is_none());

    assert_eq!(
        get_current_network_uid(&context).await?.zerostate_root_hash.as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    assert_eq!(
        context.net.network_uid.read().await.as_ref().unwrap().zerostate_root_hash.as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    // Second time in order to ensure that value wasn't broken after caching:
    assert_eq!(
        get_current_network_uid(&context).await?.zerostate_root_hash.as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    Ok(())
}

#[tokio::test]
async fn test_resolve_initial_trusted_key_block_main() -> Result<()> {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());
    let context = client.context();

    let (seq_no, root_hash) = resolve_initial_trusted_key_block(&context, 100).await?;

    assert_eq!(
        (seq_no, *root_hash.as_array()),
        INITIAL_TRUSTED_KEY_BLOCKS.get(UInt256::from_str(MAINNET_ZEROSTATE_ROOT_HASH)?.as_array()).unwrap()[0],
    );

    Ok(())
}

fn create_engine_mainnet() -> ProofHelperEngineImpl {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());
    let storage = Arc::new(InMemoryKeyValueStorage::new());
    ProofHelperEngineImpl::with_values(client.context(), storage)
}

#[tokio::test]
async fn test_metadata_storage() -> Result<()> {
    let engine = create_engine_mainnet();

    const KEY: &str = "test";
    const TEST1_VALUE: u32 = 42;
    const TEST2_VALUE: u32 = 100;
    assert!(engine.read_metadata_value_u32(KEY).await?.is_none());

    engine.write_metadata_value_u32(KEY, TEST1_VALUE).await?;
    assert_eq!(engine.read_metadata_value_u32(KEY).await?, Some(TEST1_VALUE));

    engine.update_metadata_value_u32(KEY, TEST2_VALUE, std::cmp::min).await?;
    assert_eq!(engine.read_metadata_value_u32(KEY).await?, Some(TEST1_VALUE));

    engine.update_metadata_value_u32(KEY, TEST2_VALUE, std::cmp::max).await?;
    assert_eq!(engine.read_metadata_value_u32(KEY).await?, Some(TEST2_VALUE));

    engine.update_metadata_value_u32(KEY, TEST1_VALUE, std::cmp::min).await?;
    assert_eq!(engine.read_metadata_value_u32(KEY).await?, Some(TEST1_VALUE));

    Ok(())
}

#[tokio::test]
async fn test_special_metadata_storage() -> Result<()> {
    let engine = create_engine_mainnet();

    assert_eq!(engine.read_zs_right_bound().await?, 0);

    engine.update_zs_right_bound(0).await?;
    assert_eq!(engine.read_zs_right_bound().await?, 0);

    engine.update_zs_right_bound(200).await?;
    assert_eq!(engine.read_zs_right_bound().await?, 200);

    engine.update_zs_right_bound(300).await?;
    assert_eq!(engine.read_zs_right_bound().await?, 300);

    engine.update_zs_right_bound(10).await?;
    assert_eq!(engine.read_zs_right_bound().await?, 300);


    assert_eq!(engine.read_trusted_block_right_bound(10).await?, 10);

    engine.update_trusted_block_right_bound(10, 25).await?;
    assert_eq!(engine.read_trusted_block_right_bound(10).await?, 25);

    assert_eq!(engine.read_trusted_block_right_bound(15).await?, 15);

    engine.update_trusted_block_right_bound(15, 16).await?;
    assert_eq!(engine.read_trusted_block_right_bound(15).await?, 16);

    // Ensure these values are unchanged:
    assert_eq!(engine.read_zs_right_bound().await?, 300);

    Ok(())
}

#[tokio::test]
async fn query_zerostate_boc_test() -> Result<()> {
    let engine = create_engine_mainnet();
    let zs_boc = engine.query_zerostate_boc().await?;

    let shard_state = ShardStateUnsplit::construct_from_bytes(&zs_boc)?;
    assert_eq!(shard_state.id(), "shard: -1:8000000000000000, seq_no: 0");

    Ok(())
}

#[tokio::test]
async fn query_file_hash_test() -> Result<()> {
    let engine = create_engine_mainnet();

    let file_hash_from_next = UInt256::from_str(
        &engine.query_file_hash_from_next_block(1).await?.unwrap(),
    )?;
    let file_hash_from_boc = engine.download_block_boc_and_calc_file_hash(
        "4bba527c0f5301ac01194020edb6c237158bae872348ba36b0137d523fadd864",
    ).await?;

    assert_eq!(file_hash_from_boc, file_hash_from_next);
    assert_eq!(
        file_hash_from_boc.as_hex_string(),
        "5e64ec9e1baa18b4afef021c0bca224ebf4740e227b5ddd8a0131c777a914083",
    );

    Ok(())
}

#[tokio::test]
async fn query_mc_proof_test() -> Result<()> {
    let engine = create_engine_mainnet();
    let proof_json = engine.query_mc_block_proof(1).await?;
    let proof = BlockProof::from_value(&proof_json)?;

    assert_eq!(
        proof.id().to_string(),
        "(-1:8000000000000000, 1, \
            rh 4bba527c0f5301ac01194020edb6c237158bae872348ba36b0137d523fadd864, \
            fh 5e64ec9e1baa18b4afef021c0bca224ebf4740e227b5ddd8a0131c777a914083)",
    );

    Ok(())
}

#[tokio::test]
async fn query_key_blocks_proofs_test() -> Result<()> {
    let engine = create_engine_mainnet();

    let proofs = engine.query_key_blocks_proofs(0..1000000).await?;

    assert_eq!(proofs.len(), 110);

    Ok(())
}

#[tokio::test]
async fn add_file_hashes_test() -> Result<()> {
    let engine = create_engine_mainnet();
    let mut proofs = engine.query_key_blocks_proofs(0..100000).await?;

    assert_eq!(proofs.len(), 10);

    engine.add_mc_blocks_file_hashes(&mut proofs).await?;

    for (seq_no, proof) in &proofs {
        let file_hash = engine.query_file_hash_from_next_block(*seq_no).await?.unwrap();
        assert_eq!(proof["file_hash"].as_str().unwrap(), file_hash);
    }

    Ok(())
}

#[tokio::test]
async fn mc_proofs_test() -> Result<()> {
    let engine = create_engine_mainnet();
    let storage: &InMemoryKeyValueStorage = engine.storage().in_memory();

    let proof = BlockProof::from_value(&engine.query_mc_block_proof(100000).await?)?;
    proof.check_proof(&engine).await?;

    storage.dump();

    assert_eq!(storage.count(), 1);
    assert_eq!(engine.read_zs_right_bound().await?, 0);

    let (trusted_seq_no, _trusted_root_hash) = resolve_initial_trusted_key_block(
        engine.context(), 10000000,
    ).await?;

    let proof = BlockProof::from_value(&engine.query_mc_block_proof(trusted_seq_no + 1000).await?)?;
    proof.check_proof(&engine).await?;

    storage.dump();

    assert_eq!(storage.count(), 2);
    assert_eq!(engine.read_zs_right_bound().await?, 0);
    assert_eq!(engine.read_trusted_block_right_bound(trusted_seq_no).await?, trusted_seq_no);

    Ok(())
}

#[tokio::test]
async fn extract_top_shard_block_test() -> Result<()> {
    let engine = create_engine_mainnet();
    let boc = engine.download_block_boc(
        "01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95",
    ).await?;
    let block = Block::construct_from_bytes(&boc)?;

    assert!(ProofHelperEngineImpl::extract_top_shard_block(
        &block,
        &ShardIdent::with_tagged_prefix(0, 0x8000000000000000)?,
    ).is_err());

    assert!(ProofHelperEngineImpl::extract_top_shard_block(
        &block,
        &ShardIdent::with_tagged_prefix(1, 0x2000000000000000)?,
    ).is_err());

    assert_eq!(
        ProofHelperEngineImpl::extract_top_shard_block(
            &block,
            &ShardIdent::with_tagged_prefix(0, 0x2000000000000000)?,
        )?,
        (96, UInt256::from_str("e3db85d93d5d85670c261899cf56f7ef2876e0ea95cdcfc6b7e3837998242950")?),
    );

    assert_eq!(
        ProofHelperEngineImpl::extract_top_shard_block(
            &block,
            &ShardIdent::with_tagged_prefix(0, 0xa000000000000000)?,
        )?,
        (113, UInt256::from_str("845052457398c7a2f3f5ff74ebfd2b0f6567f9ceec510593be11c900ecb26cd1")?),
    );

    Ok(())
}

#[tokio::test]
async fn query_closest_mc_block_for_shard_block_test() -> Result<()> {
    let engine = create_engine_mainnet();

    let shard = ShardIdent::with_tagged_prefix(0, 0xa000000000000000)?;

    let mut first_mc_seq_no = 100;
    assert_eq!(
        engine.query_closest_mc_block_for_shard_block(&mut first_mc_seq_no, &shard, 113).await?,
        Some(100),
    );

    first_mc_seq_no = 99;
    assert_eq!(
        engine.query_closest_mc_block_for_shard_block(&mut first_mc_seq_no, &shard, 113).await?,
        Some(99),
    );

    first_mc_seq_no = 95;
    assert_eq!(
        engine.query_closest_mc_block_for_shard_block(&mut first_mc_seq_no, &shard, 109).await?,
        Some(96),
    );

    Ok(())
}

const SHARD_BLOCK_0_A000000000000000_99_BOC: &str =
    "te6ccuECEQEAArkAABwAxADeAXACBAKgAzwDRgNYA6QECgQiBCoE9AVkBWwFcwQQEe9VqgAAACoBAgMEAqCbx6\
    mHAAAAAIQBAAAAYwAAAAACAAAAAIAAAAAAAAAAXrQHmgAAAAAG6gUAAAAAAAbqBQGyfCu9AAAAAgAAAFMAAAAAx\
    AAAAAMAAAAAAAAALgUGAhG45I37QO5rKAQHCAqKBGFR1tD1H1YJuvT+AeffTPjAYu+ZO8nWZmHsiOcBjgwT+aoe\
    0gsBG0P0h0luP4GaSgcCgYy5IjwTqZXSIYYyFU8AAgACCQoDiUoz9v0VXplWHvmULzKidbstN+VVAHO6e3HYwO8\
    S0lWq1Njxs+MjZ5qaX3Wy2UJl0E1niolxk+hX2mam0xkowcM8kUYlQA8QEACYAAAAAAas/AQAAABTB3YPJAltJD\
    aifFMLGBfGSqSuvKpr0cFEvBEi342Yag2jiOSQ5MlCwAejYCI5oU5EbgLSVSCG4LHHgFMzq5O8MwCYAAAAAAbaw\
    sEAAABiQbMSOwHpTZBEIYKXXyYQ6vfIwH2vL2gyvjJiWlTNu6MhnLL8a9sPLBZv9nO/Eo7VKACm09qERB8B5wB0\
    1Kw1zAAFAAAIAA0AEDuaygAIKEgBAWFR1tD1H1YJuvT+AeffTPjAYu+ZO8nWZmHsiOcBjgwTAAIDW5Ajr+IAAAA\
    qAgAAAACAAAAAAAAAAAAAAGMAAAAAXrQHmgAAAAAG6gUBAAAAUyALDA0BEQAAAAAAAAAAUA4AAwAQAMUAAAAAAA\
    AAAAAAAAAB////AoLhm4PAEAAAAABqz8BAAAAFMHdg8kCW0kNqJ8UwsYF8ZKpK68qmvRwUS8ESLfjZhqDaOI5JD\
    kyULAB6NgIjmhTkRuAtJVIIbgsceAUzOrk7wzgAa7BQAAAAAAAAAAAAACmAAAAAA1Z+Af//////////////////\
    ////////////////////////wAADACAAAQL5eHt0";

const SHARD_BLOCK_0_A000000000000000_101_BOC: &str =
    "te6ccuECEQEAArkAABwAxADeAXACBAKgAzwDRgNYA6QECgQiBCoE9AVkBWwFcwQQEe9VqgAAACoBAgMEAqCbx6\
    mHAAAAAIQBAAAAZQAAAAACAAAAAIAAAAAAAAAAXrQHngAAAAAHCImAAAAAAAcIiYGyfCu9AAAAAgAAAFUAAAAAx\
    AAAAAMAAAAAAAAALgUGAhG45I37QO5rKAQHCAqKBO5sAcF2Ymt7CdQz/5dUOh0WyFdnKDN0tgG6P6udwTloEhUx\
    yRwpTzXcIqlKUPMO1c8qSnjjb5j9+2wRS/PnHCoAAgACCQoDiUoz9v17zLkiB8uYo0dn+vy/FUZwk6AL+dQyhFY\
    xgp1SxSWRqliYMsW1ZyavhBdHyrATjsbtXtXllTZbMVB7VqzXoiYbQA8QEACYAAAAAAbqBQQAAABVTv2/9/Rq22\
    nN1Ir0xVEuUwWTscgsfOTgmIxFvvgKwGaYZ3Sq5GZOCOdRIpGuFelcnV52ntRlSzGxNDsLtSx6IgCYAAAAAAb5R\
    0EAAABkfIg4BEAVcwd2QRhSmkdM9cXU/uFjjWL4NSVkjAM3Z6wSgR7EbvVRzbJIhp3R55crsh2egsOSo5nYChOa\
    NiKmpwAFAAAIAA0AEDuaygAIKEgBAe5sAcF2Ymt7CdQz/5dUOh0WyFdnKDN0tgG6P6udwTloAAIDW5Ajr+IAAAA\
    qAgAAAACAAAAAAAAAAAAAAGUAAAAAXrQHngAAAAAHCImBAAAAVSALDA0BEQAAAAAAAAAAUA4AAwAQAMUAAAAAAA\
    AAAAAAAAAH////AoLwgjZAEAAAAABuoFBAAAAFVO/b/39Grbac3UivTFUS5TBZOxyCx85OCYjEW++ArAZphndKr\
    kZk4I51Eika4V6VydXnae1GVLMbE0Owu1LHoigAa7BQAAAAAAAAAAAAACqAAAAAA3UCgf//////////////////\
    ////////////////////////wAADACAAAQLlB2As";

#[tokio::test]
async fn query_shard_block_bocs_test() -> Result<()> {
    let engine = create_engine_mainnet();

    let shard = ShardIdent::with_tagged_prefix(0, 0xa000000000000000)?;

    let bocs = engine.query_shard_block_bocs(&shard, 99..102).await?;

    assert_eq!(bocs.len(), 3);
    assert_eq!(bocs[0], base64::decode(SHARD_BLOCK_0_A000000000000000_99_BOC)?);
    assert_eq!(bocs[2], base64::decode(SHARD_BLOCK_0_A000000000000000_101_BOC)?);

    Ok(())
}

#[tokio::test]
async fn check_shard_block_test() -> Result<()> {
    let engine = create_engine_mainnet();

    let boc_99 = base64::decode(SHARD_BLOCK_0_A000000000000000_99_BOC)?;
    engine.check_shard_block(&boc_99).await?;

    let boc_101 = base64::decode(SHARD_BLOCK_0_A000000000000000_101_BOC)?;
    engine.check_shard_block(&boc_101).await?;

    Ok(())
}

#[tokio::test]
async fn check_mc_proof_test() -> Result<()> {
    let engine = create_engine_mainnet();

    engine.check_mc_block_proof(
        100,
        &UInt256::from_str("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")?,
    ).await?;

    // From cache:
    engine.check_mc_block_proof(
        100,
        &UInt256::from_str("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")?,
    ).await?;

    assert!(
        engine.check_mc_block_proof(
            101,
            &UInt256::from_str("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")?,
        ).await.is_err(),
    );

    // From cache:
    assert!(
        engine.check_mc_block_proof(
            100,
            &UInt256::from_str("1111111111111111111111111111111111111111111111111111111111111111")?,
        ).await.is_err(),
    );

    Ok(())
}

async fn query_block_data(context: Arc<ClientContext>, id: &str, result: &str) -> Result<Value> {
    Ok(query_collection(
        context,
        ParamsOfQueryCollection {
            collection: "blocks".to_string(),
            result: result.to_string(),
            filter: Some(json!({
                "id": {
                    "eq": id,
                },
            })),
            limit: Some(1),
            ..Default::default()
        }
    ).await?.result.remove(0))
}

#[tokio::test]
async fn proof_block_data_test() -> Result<()> {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());

    let proof_json = query_block_data(
        client.context(),
        "5a049e5b761c1cb4bbedf0df8efb202b55a243ad194f8cb03c6e34cac48d448c",
        r#"
            id
            workchain_id
            shard
            seq_no
            gen_utime
            signatures {
                proof
                catchain_seqno
                validator_list_hash_short
                sig_weight
                signatures {
                    node_id
                    r
                    s
                }
            }
        "#
    ).await?;

    client.request_async(
        "proofs.proof_block_data",
        ParamsOfProofBlockData { block: proof_json },
    ).await?;

    let mut block_json = query_block_data(
        client.context(),
        "8bde590a572437332977e68bace66fa00f9cebac6baa57f6bf2d2f1276db2848",
        r#"
            id
            status
            status_name
            boc
            global_id
            version
            after_merge
            before_split
            after_split
            want_split
            want_merge
            key_block
            seq_no
            vert_seq_no
            gen_utime
            start_lt
            end_lt
            gen_validator_list_hash_short
            gen_catchain_seqno
            min_ref_mc_seqno
            prev_key_block_seqno
            workchain_id
            shard
            gen_software_version
            gen_software_capabilities
            prev_ref {
                end_lt
                seq_no
                root_hash
                file_hash
            }
            value_flow {
                from_prev_blk
                from_prev_blk_other {
                    currency
                    value
                }
                to_next_blk
                to_next_blk_other {
                    currency
                    value
                }
                imported
                exported
                fees_collected
                fees_imported
                created
                minted
            }
            state_update {
                old_hash
                new_hash
                old_depth
                new_depth
            }

            in_msg_descr {
                in_msg {
                    msg_id
                    cur_addr
                    next_addr
                    fwd_fee_remaining
                }
                transaction_id
                fwd_fee
                msg_type
                msg_type_name
            }
            out_msg_descr {
                import_block_lt
                imported {
                    fwd_fee
                    ihr_fee
                    in_msg {
                        cur_addr
                        fwd_fee_remaining
                        msg_id
                        next_addr
                        __typename
                    }
                    msg_id
                    msg_type
                    msg_type_name
                    out_msg {
                        cur_addr
                        fwd_fee_remaining
                        msg_id
                        next_addr
                        __typename
                    }
                    proof_created
                    proof_delivered
                    transaction_id
                    transit_fee
                    __typename
                }
                msg_env_hash
                msg_id
                msg_type
                msg_type_name
                next_addr_pfx
                next_workchain
                out_msg {
                    cur_addr
                    fwd_fee_remaining
                    msg_id
                    next_addr
                    __typename
                }
                reimport {
                    fwd_fee
                    ihr_fee
                    in_msg {
                        cur_addr
                        fwd_fee_remaining
                        msg_id
                        next_addr
                        __typename
                    }
                    msg_id
                    msg_type
                    msg_type_name
                    out_msg {
                        cur_addr
                        fwd_fee_remaining
                        msg_id
                        next_addr
                        __typename
                    }
                    proof_created
                    proof_delivered
                    transaction_id
                    transit_fee
                    __typename
                }
                transaction_id
                __typename
            }
            account_blocks {
                account_addr
                transactions {
                    lt
                    total_fees
                    total_fees_other {
                        currency
                        value
                        __typename
                    }
                    transaction_id
                    __typename
                }
                old_hash
                new_hash
                tr_count
            }
            tr_count
            rand_seed
            created_by
            master {
                shard_hashes {
                    workchain_id
                    shard
                    descr {
                        seq_no
                        reg_mc_seqno
                        start_lt
                        end_lt
                        root_hash
                        file_hash
                        before_split
                        before_merge
                        want_split
                        want_merge
                        nx_cc_updated
                        gen_utime
                        next_catchain_seqno
                        next_validator_shard
                        min_ref_mc_seqno
                        flags
                        fees_collected
                        funds_created
                    }
                }
                min_shard_gen_utime
                max_shard_gen_utime
                shard_fees {
                    workchain_id
                    shard
                    fees
                    create
                }
                prev_blk_signatures {
                    node_id
                    s
                    r
                }
                recover_create_msg {
                    in_msg {
                        cur_addr
                        fwd_fee_remaining
                        msg_id
                        next_addr
                        __typename
                    }
                    msg_id
                    transaction_id
                    fwd_fee
                    msg_type
                    msg_type_name
                    ihr_fee
                    out_msg {
                        cur_addr
                        fwd_fee_remaining
                        msg_id
                        next_addr
                        __typename
                    }
                    proof_created
                    proof_delivered
                    transit_fee
                }
            }
		"#,
    ).await?;

    client.request_async(
        "proofs.proof_block_data",
        ParamsOfProofBlockData { block: block_json.clone() },
    ).await?;

    block_json["boc"] = Value::Null;

    client.request_async(
        "proofs.proof_block_data",
        ParamsOfProofBlockData { block: block_json.clone() },
    ).await?;

    block_json["boc"] = SHARD_BLOCK_0_A000000000000000_99_BOC.into();

    assert!(
        client.request_async::<_, ()>(
            "proofs.proof_block_data",
            ParamsOfProofBlockData { block: block_json.clone() },
        ).await
            .is_err()
    );

    block_json["boc"] = Value::Null;
    block_json["prev_ref"]["root_hash"] = "0000000000000000000000000000000000000000000000000000000000000000".into();

    assert!(
        client.request_async::<_, ()>(
            "proofs.proof_block_data",
            ParamsOfProofBlockData { block: block_json },
        ).await
            .is_err()
    );

    Ok(())
}
