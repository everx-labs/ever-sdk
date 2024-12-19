use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::bail;
use graphql_parser::schema::{Definition, ObjectType, Type, TypeDefinition};
use serde_json::Value;
use ever_block::{
    BinTreeType, Block, BlockIdExt, Deserializable, InRefValue, ShardHashes, ShardIdent,
    ShardStateUnsplit, MASTERCHAIN_ID,
};
use ever_block::{Result, UInt256};

use crate::client::storage::InMemoryKeyValueStorage;
use crate::net::{query_collection, ParamsOfQueryCollection};
use crate::proofs::engine::ProofHelperEngineImpl;
use crate::proofs::validators::{
    calc_subset_for_workchain, calc_workchain_id, calc_workchain_id_by_adnl_id,
};
use crate::proofs::{
    is_transaction_refers_to_message, message_get_required_data, proof_message_data,
    proof_transaction_data, resolve_initial_trusted_key_block, transaction_get_required_data,
    BlockProof, ParamsOfProofBlockData, ParamsOfProofMessageData, ParamsOfProofTransactionData,
    INITIAL_TRUSTED_KEY_BLOCKS,
};
use crate::tests::TestClient;
use crate::ClientContext;

const GQL_SCHEMA: &str = include_str!("data/schema.graphql");

#[test]
fn test_check_master_blocks_proof() -> Result<()> {
    let key_block_proof = BlockProof::read_from_file(
        "src/proofs/tests/data/test_master_block_proof/key_proof__3082181",
    )?;

    for seq_no in 3082182..=3082200 {
        let block_proof = BlockProof::read_from_file(format!(
            "src/proofs/tests/data/test_master_block_proof/proof__{}",
            seq_no
        ))?;
        let (virt_block, virt_block_info) = block_proof.pre_check_block_proof()?;
        block_proof.check_with_prev_key_block_proof(
            &key_block_proof,
            &virt_block,
            &virt_block_info,
        )?;
    }

    Ok(())
}

#[test]
fn test_check_master_blocks_proof_shuffle() -> Result<()> {
    let key_block_proof = BlockProof::read_from_file(
        "src/proofs/tests/data/test_master_block_proof_shuffle/key_proof__3236530",
    )?;

    for seq_no in 3236531..=3236550 {
        let block_proof = BlockProof::read_from_file(format!(
            "src/proofs/tests/data/test_master_block_proof_shuffle/proof__{}",
            seq_no
        ))?;

        let (virt_block, virt_block_info) = block_proof.pre_check_block_proof()?;
        block_proof.check_with_prev_key_block_proof(
            &key_block_proof,
            &virt_block,
            &virt_block_info,
        )?;
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
    assert!(
        config.prev_validator_set_present()?,
        "key block must be after elections"
    );

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
        let subset = calc_subset_for_workchain(
            &vset,
            config,
            &cc_config,
            ever_block::SHARD_FULL,
            workchain_id,
            cc_seqno,
            0.into(),
        )?;
        assert_eq!(subset.0.len(), 7);
        subset.0.iter().enumerate().for_each(|(_i, descr)| {
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
                let block_id = BlockIdExt::with_params(
                    shard_ident,
                    shard_descr.seq_no,
                    shard_descr.root_hash,
                    shard_descr.file_hash,
                );
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
            MASTERCHAIN_ID => vec![BlockIdExt::with_params(
                ShardIdent::masterchain(),
                0,
                Default::default(),
                Default::default(),
            )],
            workchain_id => get_top_blocks(custom.shards(), &[workchain_id])?,
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
                Default::default(),
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

fn mainnet_config() -> Value {
    json!({
        "network": {
            "endpoints": [TestClient::with_project("mainnet.evercloud.dev")],
        }
    })
}

#[tokio::test]
async fn test_query_current_network_zerostate_root_hash() -> Result<()> {
    let client = TestClient::new_with_config(mainnet_config());

    assert_eq!(
        client
            .context()
            .net
            .query_current_network_uid()
            .await?
            .zerostate_root_hash
            .as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    Ok(())
}

#[tokio::test]
async fn test_get_current_network_zerostate_root_hash() -> Result<()> {
    let client = TestClient::new_with_config(mainnet_config());
    let context = client.context();

    assert!(context.net.network_uid.read().await.is_none());

    assert_eq!(
        context
            .net
            .get_current_network_uid()
            .await?
            .zerostate_root_hash
            .as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    assert_eq!(
        context
            .net
            .network_uid
            .read()
            .await
            .as_ref()
            .unwrap()
            .zerostate_root_hash
            .as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    // Second time in order to ensure that value wasn't broken after caching:
    assert_eq!(
        context
            .net
            .get_current_network_uid()
            .await?
            .zerostate_root_hash
            .as_hex_string(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    Ok(())
}

#[tokio::test]
async fn test_resolve_initial_trusted_key_block_main() -> Result<()> {
    let client = TestClient::new_with_config(mainnet_config());
    let context = client.context();

    let (seq_no, root_hash) = resolve_initial_trusted_key_block(&context, 100).await?;

    assert_eq!(
        (seq_no, *root_hash.as_array()),
        INITIAL_TRUSTED_KEY_BLOCKS
            .get(UInt256::from_str(MAINNET_ZEROSTATE_ROOT_HASH)?.as_array())
            .unwrap()[0],
    );

    Ok(())
}

fn create_engine_mainnet() -> ProofHelperEngineImpl {
    let client = TestClient::new_with_config(mainnet_config());
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
    assert_eq!(
        engine.read_metadata_value_u32(KEY).await?,
        Some(TEST1_VALUE)
    );

    engine
        .update_metadata_value_u32(KEY, TEST2_VALUE, std::cmp::min)
        .await?;
    assert_eq!(
        engine.read_metadata_value_u32(KEY).await?,
        Some(TEST1_VALUE)
    );

    engine
        .update_metadata_value_u32(KEY, TEST2_VALUE, std::cmp::max)
        .await?;
    assert_eq!(
        engine.read_metadata_value_u32(KEY).await?,
        Some(TEST2_VALUE)
    );

    engine
        .update_metadata_value_u32(KEY, TEST1_VALUE, std::cmp::min)
        .await?;
    assert_eq!(
        engine.read_metadata_value_u32(KEY).await?,
        Some(TEST1_VALUE)
    );

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
async fn test_query_zerostate_boc() -> Result<()> {
    let engine = create_engine_mainnet();
    let zs_boc = engine.query_zerostate_boc().await?;

    let shard_state = ShardStateUnsplit::construct_from_bytes(&zs_boc)?;
    assert_eq!(shard_state.id(), "shard: -1:8000000000000000, seq_no: 0");

    Ok(())
}

#[tokio::test]
async fn test_query_file_hash() -> Result<()> {
    let engine = create_engine_mainnet();

    let file_hash_from_next =
        UInt256::from_str(&engine.query_file_hash_from_next_block(1).await?.unwrap())?;
    let file_hash_from_boc = engine
        .download_block_boc_and_calc_file_hash(
            "4bba527c0f5301ac01194020edb6c237158bae872348ba36b0137d523fadd864",
        )
        .await?;

    assert_eq!(file_hash_from_boc, file_hash_from_next);
    assert_eq!(
        file_hash_from_boc.as_hex_string(),
        "5e64ec9e1baa18b4afef021c0bca224ebf4740e227b5ddd8a0131c777a914083",
    );

    Ok(())
}

#[tokio::test]
async fn test_query_mc_proof() -> Result<()> {
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
async fn test_query_key_blocks_proofs() -> Result<()> {
    let engine = create_engine_mainnet();

    let proofs = engine.query_key_blocks_proofs(0..1000000).await?;

    assert_eq!(proofs.len(), 110);

    Ok(())
}

#[tokio::test]
async fn test_add_file_hashes() -> Result<()> {
    let engine = create_engine_mainnet();
    let mut proofs = engine.query_key_blocks_proofs(0..100000).await?;

    assert_eq!(proofs.len(), 10);

    engine.add_mc_blocks_file_hashes(&mut proofs).await?;

    for (seq_no, proof) in &proofs {
        let file_hash = engine
            .query_file_hash_from_next_block(*seq_no)
            .await?
            .unwrap();
        assert_eq!(proof["file_hash"].as_str().unwrap(), file_hash);
    }

    Ok(())
}

#[tokio::test]
async fn test_mc_proofs() -> Result<()> {
    let engine = create_engine_mainnet();
    let storage: &InMemoryKeyValueStorage = engine.storage().in_memory();

    let proof = BlockProof::from_value(&engine.query_mc_block_proof(100000).await?)?;
    proof.check_proof(&engine).await?;

    storage.dump();

    assert_eq!(storage.count(), 1);
    assert_eq!(engine.read_zs_right_bound().await?, 0);

    let (trusted_seq_no, _trusted_root_hash) =
        resolve_initial_trusted_key_block(engine.context(), 10000000).await?;

    let proof = BlockProof::from_value(&engine.query_mc_block_proof(trusted_seq_no + 1000).await?)?;
    proof.check_proof(&engine).await?;

    storage.dump();

    assert_eq!(storage.count(), 2);
    assert_eq!(engine.read_zs_right_bound().await?, 0);
    assert_eq!(
        engine
            .read_trusted_block_right_bound(trusted_seq_no)
            .await?,
        trusted_seq_no
    );

    Ok(())
}

#[tokio::test]
async fn test_extract_top_shard_block() -> Result<()> {
    let engine = create_engine_mainnet();
    let boc = engine
        .download_block_boc("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")
        .await?;
    let block = Block::construct_from_bytes(&boc)?;

    assert!(ProofHelperEngineImpl::extract_top_shard_block(
        &block,
        &ShardIdent::with_tagged_prefix(0, 0x8000000000000000)?,
    )
    .is_err());

    assert!(ProofHelperEngineImpl::extract_top_shard_block(
        &block,
        &ShardIdent::with_tagged_prefix(1, 0x2000000000000000)?,
    )
    .is_err());

    assert_eq!(
        ProofHelperEngineImpl::extract_top_shard_block(
            &block,
            &ShardIdent::with_tagged_prefix(0, 0x2000000000000000)?,
        )?,
        (
            96,
            UInt256::from_str("e3db85d93d5d85670c261899cf56f7ef2876e0ea95cdcfc6b7e3837998242950")?
        ),
    );

    assert_eq!(
        ProofHelperEngineImpl::extract_top_shard_block(
            &block,
            &ShardIdent::with_tagged_prefix(0, 0xa000000000000000)?,
        )?,
        (
            113,
            UInt256::from_str("845052457398c7a2f3f5ff74ebfd2b0f6567f9ceec510593be11c900ecb26cd1")?
        ),
    );

    Ok(())
}

#[tokio::test]
async fn test_query_closest_mc_block_for_shard_block() -> Result<()> {
    let engine = create_engine_mainnet();

    let shard = ShardIdent::with_tagged_prefix(0, 0xa000000000000000)?;

    let mut first_mc_seq_no = 100;
    assert_eq!(
        engine
            .query_closest_mc_block_for_shard_block(&mut first_mc_seq_no, &shard, 113)
            .await?,
        Some(100),
    );

    first_mc_seq_no = 99;
    assert_eq!(
        engine
            .query_closest_mc_block_for_shard_block(&mut first_mc_seq_no, &shard, 113)
            .await?,
        Some(99),
    );

    first_mc_seq_no = 95;
    assert_eq!(
        engine
            .query_closest_mc_block_for_shard_block(&mut first_mc_seq_no, &shard, 109)
            .await?,
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
async fn test_query_shard_block_bocs() -> Result<()> {
    let engine = create_engine_mainnet();

    let shard = ShardIdent::with_tagged_prefix(0, 0xa000000000000000)?;

    let bocs = engine.query_shard_block_bocs(&shard, 99..102).await?;

    assert_eq!(bocs.len(), 3);
    assert_eq!(
        bocs[0],
        base64::decode(SHARD_BLOCK_0_A000000000000000_99_BOC)?
    );
    assert_eq!(
        bocs[2],
        base64::decode(SHARD_BLOCK_0_A000000000000000_101_BOC)?
    );

    Ok(())
}

#[tokio::test]
async fn test_check_shard_block() -> Result<()> {
    let engine = create_engine_mainnet();

    let boc_99 = base64::decode(SHARD_BLOCK_0_A000000000000000_99_BOC)?;
    engine.check_shard_block(&boc_99).await?;

    let boc_101 = base64::decode(SHARD_BLOCK_0_A000000000000000_101_BOC)?;
    engine.check_shard_block(&boc_101).await?;

    Ok(())
}

#[tokio::test]
async fn test_check_mc_proof() -> Result<()> {
    let engine = create_engine_mainnet();

    engine
        .check_mc_block_proof(
            100,
            &UInt256::from_str("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")?,
        )
        .await?;

    // From cache:
    engine
        .check_mc_block_proof(
            100,
            &UInt256::from_str("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")?,
        )
        .await?;

    assert!(engine
        .check_mc_block_proof(
            101,
            &UInt256::from_str("01872c85facaa85405518a759dfac2625bc94b9e85b965cf3875d2331db9ad95")?,
        )
        .await
        .is_err(),);

    // From cache:
    assert!(engine
        .check_mc_block_proof(
            100,
            &UInt256::from_str("1111111111111111111111111111111111111111111111111111111111111111")?,
        )
        .await
        .is_err(),);

    Ok(())
}

async fn query_data(
    context: Arc<ClientContext>,
    collection: &str,
    id: &str,
    result: &str,
) -> Result<Value> {
    Ok(query_collection(
        context,
        ParamsOfQueryCollection {
            collection: collection.to_string(),
            result: result.to_string(),
            filter: Some(json!({
                "id": {
                    "eq": id,
                },
            })),
            limit: Some(1),
            ..Default::default()
        },
    )
    .await?
    .result
    .remove(0))
}

async fn query_block_data(context: Arc<ClientContext>, id: &str, result: &str) -> Result<Value> {
    query_data(context, "blocks", id, result).await
}

async fn query_transaction_data(
    context: Arc<ClientContext>,
    id: &str,
    result: &str,
) -> Result<Value> {
    query_data(context, "transactions", id, result).await
}

async fn query_message_data(context: Arc<ClientContext>, id: &str, result: &str) -> Result<Value> {
    query_data(context, "messages", id, result).await
}

fn resolve_type_name(typ: &Type<String>) -> String {
    match typ {
        Type::NamedType(ref name) => name.clone(),
        Type::ListType(typ) => resolve_type_name(typ.as_ref()),
        Type::NonNullType(typ) => resolve_type_name(typ.as_ref()),
    }
}

fn print_object_type(
    object_type: &ObjectType<String>,
    known_types: &HashMap<String, &ObjectType<String>>,
    path: String,
    output: &mut String,
) {
    for i in 0..object_type.fields.len() {
        let field = &object_type.fields[i];
        if field
            .arguments
            .iter()
            .find(|arg| arg.name == "when")
            .is_some()
        {
            continue;
        }
        let type_name = resolve_type_name(&field.field_type);
        let field_ident = format!("{}:{}", field.name, type_name);
        if path.contains(&format!("/{}/", field_ident)) {
            continue;
        }
        if i != 0 {
            output.push(' ');
        }
        output.push_str(&field.name);
        if let Some(typ) = known_types.get(&type_name) {
            output.push('{');
            print_object_type(
                typ,
                known_types,
                format!("{}{}/", path, field_ident),
                output,
            );
            output.push('}');
        }
    }
}

fn gen_full_schema_query(object_type: &str) -> Result<String> {
    let schema = graphql_parser::parse_schema::<String>(GQL_SCHEMA)?;

    let mut known_types = HashMap::new();
    for definition in schema.definitions.iter() {
        if let Definition::TypeDefinition(TypeDefinition::Object(obj_type)) = definition {
            known_types.insert(obj_type.name.to_string(), obj_type);
        }
    }

    for definition in schema.definitions.iter() {
        if let Definition::TypeDefinition(TypeDefinition::Object(obj_type)) = definition {
            if obj_type.name == object_type {
                let mut output = String::new();
                print_object_type(
                    obj_type,
                    &known_types,
                    format!("/{}", obj_type.name),
                    &mut output,
                );
                return Ok(output);
            }
        }
    }

    bail!("Object type is not found in the schema: {}", object_type)
}

#[tokio::test]
async fn test_proof_block_data() -> Result<()> {
    let client = TestClient::new_with_config(mainnet_config());

    let mut block_json = query_block_data(
        client.context(),
        "8bde590a572437332977e68bace66fa00f9cebac6baa57f6bf2d2f1276db2848",
        &gen_full_schema_query("Block")?,
    )
    .await?;

    let _: () = client
        .request_async(
            "proofs.proof_block_data",
            ParamsOfProofBlockData {
                block: block_json.clone(),
            },
        )
        .await?;

    block_json["boc"] = Value::Null;

    let _: () = client
        .request_async(
            "proofs.proof_block_data",
            ParamsOfProofBlockData {
                block: block_json.clone(),
            },
        )
        .await?;

    block_json["boc"] = SHARD_BLOCK_0_A000000000000000_99_BOC.into();

    assert!(client
        .request_async::<_, ()>(
            "proofs.proof_block_data",
            ParamsOfProofBlockData {
                block: block_json.clone()
            },
        )
        .await
        .is_err());

    block_json["boc"] = Value::Null;
    block_json["prev_ref"]["root_hash"] =
        "0000000000000000000000000000000000000000000000000000000000000000".into();

    assert!(client
        .request_async::<_, ()>(
            "proofs.proof_block_data",
            ParamsOfProofBlockData { block: block_json },
        )
        .await
        .is_err());

    let proof_json = query_block_data(
        client.context(),
        "5a049e5b761c1cb4bbedf0df8efb202b55a243ad194f8cb03c6e34cac48d448c",
        r#"
            id
            workchain_id
            shard
            seq_no
            gen_utime
            start_lt
            end_lt
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
        "#,
    )
    .await?;

    assert!(client
        .request_async::<_, ()>(
            "proofs.proof_block_data",
            ParamsOfProofBlockData { block: proof_json },
        )
        .await
        .is_err());

    let decimal_fields = r#"
        id
        boc
        account_blocks {
            transactions {
                lt(format:DEC) total_fees(format:DEC) total_fees_other{value(format:DEC)}
            }
        }
        end_lt(format:DEC)
        gen_software_capabilities(format:DEC)
        in_msg_descr {
            fwd_fee(format:DEC)
            ihr_fee(format:DEC)
            in_msg {fwd_fee_remaining(format:DEC)}
            out_msg {fwd_fee_remaining(format:DEC)}
            transit_fee(format:DEC)}
        master {
            config {
              p14 {basechain_block_fee(format:DEC) masterchain_block_fee(format:DEC)}
              p17 {max_stake(format:DEC) min_stake(format:DEC) min_total_stake(format:DEC)}
              p18 {
                bit_price_ps(format:DEC)
                cell_price_ps(format:DEC)
                mc_bit_price_ps(format:DEC)
                mc_cell_price_ps(format:DEC)
              }
              p20 {
                block_gas_limit(format:DEC)
                delete_due_limit(format:DEC)
                flat_gas_limit(format:DEC)
                flat_gas_price(format:DEC)
                freeze_due_limit(format:DEC)
                gas_credit(format:DEC)
                gas_limit(format:DEC)
                gas_price(format:DEC)
                special_gas_limit(format:DEC)
              }
              p21 {
                block_gas_limit(format:DEC)
                delete_due_limit(format:DEC)
                flat_gas_limit(format:DEC)
                flat_gas_price(format:DEC)
                freeze_due_limit(format:DEC)
                gas_credit(format:DEC)
                gas_limit(format:DEC)
                gas_price(format:DEC)
                special_gas_limit(format:DEC)
              }
              p24 {bit_price(format:DEC) cell_price(format:DEC) lump_price(format:DEC)}
              p25 {bit_price(format:DEC) cell_price(format:DEC) lump_price(format:DEC)}
              p32 {list {weight(format:DEC)} total_weight(format:DEC)}
              p33 {list {weight(format:DEC)} total_weight(format:DEC)}
              p34 {list {weight(format:DEC)} total_weight(format:DEC)}
              p35 {list {weight(format:DEC)} total_weight(format:DEC)}
              p36 {list {weight(format:DEC)} total_weight(format:DEC)}
              p37 {list {weight(format:DEC)} total_weight(format:DEC)}
              p8 {capabilities(format:DEC)}
            }
            recover_create_msg {
                fwd_fee(format:DEC)
                ihr_fee(format:DEC)
                in_msg {fwd_fee_remaining(format:DEC)}
                out_msg {fwd_fee_remaining(format:DEC)}
                transit_fee(format:DEC)
            }
            shard_fees {
                create(format:DEC)
                create_other {value(format:DEC)} fees(format:DEC) fees_other {value(format:DEC)}
            }
            shard_hashes {
                descr {
                    end_lt(format:DEC)
                    fees_collected(format:DEC)
                    fees_collected_other {value(format:DEC)}
                    funds_created(format:DEC)
                    funds_created_other {value(format:DEC)} start_lt(format:DEC)
                }
            }
        }
        master_ref {end_lt(format:DEC)}
        out_msg_descr {
            import_block_lt(format:DEC)
            imported {
                fwd_fee(format:DEC)
                ihr_fee(format:DEC)
                in_msg {fwd_fee_remaining(format:DEC)}
                out_msg {fwd_fee_remaining(format:DEC)}
                transit_fee(format:DEC)
            }
            next_addr_pfx(format:DEC)
            out_msg {fwd_fee_remaining(format:DEC)}
            reimport {
                fwd_fee(format:DEC)
                ihr_fee(format:DEC)
                in_msg {fwd_fee_remaining(format:DEC)}
                out_msg {fwd_fee_remaining(format:DEC)}
                transit_fee(format:DEC)
            }
        }
        prev_alt_ref {end_lt(format:DEC)}
        prev_ref {end_lt(format:DEC)}
        prev_vert_alt_ref {end_lt(format:DEC)}
        prev_vert_ref{end_lt(format:DEC)}
        start_lt(format:DEC)
        value_flow {
            created(format:DEC)
            created_other {value(format:DEC)}
            exported exported_other {value(format:DEC)}
            fees_collected(format:DEC)
            fees_collected_other {value(format:DEC)}
            fees_imported(format:DEC)
            fees_imported_other {value(format:DEC)}
            from_prev_blk(format:DEC)
            from_prev_blk_other {value(format:DEC)}
            imported(format:DEC)
            imported_other {value(format:DEC)}
            minted(format:DEC)
            minted_other {value(format:DEC)}
            to_next_blk(format:DEC)
            to_next_blk_other {value(format:DEC)}
        }
    "#;

    // Masterchain block
    let block_json = query_block_data(
        client.context(),
        "9eee20c3a93ca93928d7dc4bbbe6570c492d09077f13ebf7b2f68f9e2e176433",
        decimal_fields,
    )
    .await?;

    let _: () = client
        .request_async(
            "proofs.proof_block_data",
            ParamsOfProofBlockData {
                block: block_json.clone(),
            },
        )
        .await?;

    // Shardchain block
    let block_json = query_block_data(
        client.context(),
        "b38d6bdb4fab0e52a9165fe65aa373520ae8c7e422f93f20c9a2a5c8016d5e7d",
        decimal_fields,
    )
    .await?;

    client
        .request_async(
            "proofs.proof_block_data",
            ParamsOfProofBlockData {
                block: block_json.clone(),
            },
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_transaction_get_required_data() -> Result<()> {
    const ID: &'static str = "5b532e2ec17ac84b4efa92703192368dd4ed8a2729f2be2b0ee4e0665368f7c0";
    const BOC: &'static str = "\
            te6ccgECBgEAATMAA69wT2TGr7/z3RDYumcHeQrJZw1UDzepRIsDN7qmpakqysAAAR0tIeN4FanEM9ilnr+FQpc\
            mlTEG3AXJ47njjdUvtmEBGza8vWswAAEdLSDvVDYWPE5wABQIBQQBAgUgMDQDAgBpYAAAAJYAAAAEAAYAAAAAAA\
            UZroTxe4+LIgJql1/1Xxqxn95KdodE0heN+mO7Uz4QekCQJrwAoEJmUBfXhAAAAAAAAAAAADAAAAAAAAAAAAAAA\
            AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIJyi1oFnzCrRTpl\
            ZW9pdoaxTsmbuXQ36fewBok3KaF+nc49AloFgjqbOapl+vKCOKiovtM8HJflgF0zlY3Eah3ZxAABIA==";

    async fn test(engine: &ProofHelperEngineImpl, transaction_json: Value) -> Result<()> {
        let (id, block_id, boc, transaction) =
            transaction_get_required_data(&engine, &transaction_json).await?;

        assert_eq!(id.as_hex_string(), ID,);
        assert_eq!(
            block_id,
            "eb7c28f1d301dff2d6ec899fb5ee18d9478f397b10c16a6f6aabb6535686266e"
        );
        assert_eq!(boc, base64::decode(BOC)?);
        assert_eq!(transaction.logical_time(), 0x11d2d21e3781);

        Ok(())
    }

    let engine = create_engine_mainnet();

    test(
        &engine,
        json!({
            "id": ID,
        }),
    )
    .await?;

    test(
        &engine,
        json!({
            "boc": BOC,
        }),
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_proof_transaction_data() -> Result<()> {
    let client = TestClient::new_with_config(mainnet_config());

    let transaction_json = query_transaction_data(
        client.context(),
        "0c7e395e8eb14c173d2dde7189200f28787a05df1fa188b19224f6e19a439dc6",
        &gen_full_schema_query("Transaction")?,
    )
    .await?;

    proof_transaction_data(
        client.context(),
        ParamsOfProofTransactionData {
            transaction: transaction_json,
        },
    )
    .await?;

    let transaction_json = query_transaction_data(
        client.context(),
        "0c7e395e8eb14c173d2dde7189200f28787a05df1fa188b19224f6e19a439dc6",
        r#"
            id
            boc
            action {total_action_fees(format:DEC) total_fwd_fees(format:DEC)}
            balance_delta(format:DEC)
            balance_delta_other {value(format:DEC)}
            bounce {fwd_fees(format:DEC) msg_fees(format:DEC) req_fwd_fees(format:DEC)}
            compute {gas_fees(format:DEC) gas_limit(format:DEC) gas_used(format:DEC)}
            credit {credit(format:DEC) credit_other {value(format:DEC)} due_fees_collected(format:DEC)}
            ext_in_msg_fee(format:DEC)
            lt(format:DEC)
            prev_trans_lt(format:DEC)
            storage {storage_fees_collected(format:DEC) storage_fees_due(format:DEC)}
            total_fees(format:DEC)
            total_fees_other {value(format:DEC)}
        "#,
    ).await?;

    proof_transaction_data(
        client.context(),
        ParamsOfProofTransactionData {
            transaction: transaction_json,
        },
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_message_get_required_data() -> Result<()> {
    async fn test(
        engine: &ProofHelperEngineImpl,
        message_json: Value,
        message_id: &str,
        message_boc: &str,
        message_trans_id: &str,
        dst_account_address: Option<&str>,
    ) -> Result<()> {
        let (id, trans_id, boc, message) =
            message_get_required_data(&engine, &message_json).await?;

        assert_eq!(id.as_hex_string(), message_id,);
        assert_eq!(trans_id, message_trans_id);
        assert_eq!(boc, base64::decode(message_boc)?);
        assert_eq!(
            message.dst_ref().map(|addr| addr.to_string()),
            dst_account_address.map(|str| str.to_string()),
        );

        Ok(())
    }

    async fn tests(
        engine: &ProofHelperEngineImpl,
        message_id: &str,
        message_boc: &str,
        trans_id: &str,
        dst_account_address: Option<&str>,
    ) -> Result<()> {
        test(
            &engine,
            json!({
                "id": message_id,
            }),
            message_id,
            message_boc,
            trans_id,
            dst_account_address,
        )
        .await?;

        test(
            &engine,
            json!({
                "boc": message_boc,
            }),
            message_id,
            message_boc,
            trans_id,
            dst_account_address,
        )
        .await
    }

    let engine = create_engine_mainnet();

    tests(
        &engine,
        "228a430e2df4c7ec46f493a0add954cb54dc387ab140a779576988fc603ac699",
        "\
            te6ccgEBAQEAWAAAq2n+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAE/zMzMzMzMzMzMzMzMzMzMzMz\
            MzMzMzMzMzMzMzMzMzMzSH9xlJAAAACOlpFrzgMLHidRA",
        "2658e6c5371f73468b6d4afaaf2f681f8f39e256dc2f5b66362a9a8a002177a9",
        Some("-1:3333333333333333333333333333333333333333333333333333333333333333"),
    )
    .await?;

    tests(
        &engine,
        "420cefa19e4daf01ebe5db21c1ece04eee8bb457ca76680385c70b652596887f",
        "\
            te6ccgEBAQEAPQAAdeAG+RbNyA4dMePcEfXbfAecY743sLt3ixyG2hZOh4AloJAAACOlpDxvCMLHidBJjsFmgAA\
            AAAAAAABA",
        "0c7e395e8eb14c173d2dde7189200f28787a05df1fa188b19224f6e19a439dc6",
        None,
    )
    .await?;

    Ok(())
}

#[test]
fn test_is_transaction_refers_to_message() {
    let id = Value::String(
        "5b532e2ec17ac84b4efa92703192368dd4ed8a2729f2be2b0ee4e0665368f7c0".to_owned(),
    );
    let json = json!({});
    assert!(!is_transaction_refers_to_message(&json, &id));

    let json = json!({
        "in_msg": "aaa",
        "out_msgs": ["bbb", "ccc"],
    });

    assert!(!is_transaction_refers_to_message(&json, &id));

    let json = json!({
        "in_msg": id,
        "out_msgs": [],
    });

    assert!(is_transaction_refers_to_message(&json, &id));

    let json = json!({
        "in_msg": "aaa",
        "out_msgs": ["aaa", "bbb", id, "ddd"],
    });

    assert!(is_transaction_refers_to_message(&json, &id));
}

#[tokio::test]
async fn test_proof_message_data() -> Result<()> {
    let client = TestClient::new_with_config(mainnet_config());

    let message_json = query_message_data(
        client.context(),
        "4a9389e2fa34a83db0c814674bc4c7569fd3e92042289e2b2d4802231ecabec9",
        &gen_full_schema_query("Message")?,
    )
    .await?;

    proof_message_data(
        client.context(),
        ParamsOfProofMessageData {
            message: message_json,
        },
    )
    .await?;

    let message_json = query_message_data(
        client.context(),
        "4a9389e2fa34a83db0c814674bc4c7569fd3e92042289e2b2d4802231ecabec9",
        r#"
            id
            boc
            created_lt(format:DEC)
            fwd_fee(format:DEC)
            ihr_fee(format:DEC)
            import_fee(format:DEC)
            value(format:DEC)
            value_other{value(format:DEC)}
        "#,
    )
    .await?;

    proof_message_data(
        client.context(),
        ParamsOfProofMessageData {
            message: message_json,
        },
    )
    .await?;

    Ok(())
}
