use std::sync::Arc;

use serde_json::Value;
use ton_block::{BinTreeType, Block, BlockIdExt, Deserializable, InRefValue, MASTERCHAIN_ID, ShardHashes, ShardIdent, ShardStateUnsplit};
use ton_types::{Result, UInt256};

use crate::proofs::{BlockProof, get_current_network_uid, INITIAL_TRUSTED_KEY_BLOCKS, query_current_network_uid, resolve_initial_trusted_key_block};
use crate::proofs::engine::ProofHelperEngineImpl;
use crate::proofs::storage::InMemoryProofStorage;
use crate::proofs::validators::{calc_subset_for_workchain, calc_workchain_id, calc_workchain_id_by_adnl_id};
use crate::tests::TestClient;

#[test]
fn test_check_master_blocks_proof() -> Result<()> {
    let key_block_proof = BlockProof::read_from_file(
        "src/proofs/tests/data/test_master_block_proof/key_proof__3082181"
    )?;

    for seq_no in 3082182..=3082200 {
        let block_proof = BlockProof::read_from_file(
            format!("src/proofs/tests/data/test_master_block_proof/proof__{}", seq_no)
        )?;
        block_proof.check_with_prev_key_block_proof(&key_block_proof)?;
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

        block_proof.check_with_prev_key_block_proof(&key_block_proof)?;
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
        query_current_network_uid(client.context()).await?.zerostate_root_hash.as_str(),
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
        get_current_network_uid(&context).await?.zerostate_root_hash.as_str(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    assert_eq!(
        context.net.network_uid.read().await.as_ref().unwrap().zerostate_root_hash.as_str(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    // Second time in order to ensure that value wasn't broken after caching:
    assert_eq!(
        get_current_network_uid(&context).await?.zerostate_root_hash.as_str(),
        MAINNET_ZEROSTATE_ROOT_HASH,
    );

    Ok(())
}

#[tokio::test]
async fn test_resolve_initial_trusted_key_block_main() -> Result<()> {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());
    let context = client.context();

    let trusted_block_id = resolve_initial_trusted_key_block(&context).await?;

    assert_eq!(
        trusted_block_id,
        &INITIAL_TRUSTED_KEY_BLOCKS.get(&MAINNET_ZEROSTATE_ROOT_HASH.to_string())
            .unwrap()
            .trusted_key_block,
    );

    Ok(())
}

#[test]
fn test_gen_storage_key() {
    let network_uid = crate::client::NetworkUID {
        zerostate_root_hash:
            "0123456790abcdef0123456790abcdef0123456790abcdef0123456790abcdef".to_string(),
        first_master_block_root_hash:
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string(),
    };
    assert_eq!(
        ProofHelperEngineImpl::<InMemoryProofStorage>::gen_storage_key(&network_uid, "test"),
        "01234567/abcdef01/test",
    );
}

fn create_engine_mainnet() -> ProofHelperEngineImpl<InMemoryProofStorage> {
    let client = TestClient::new_with_config(MAINNET_CONFIG.clone());
    let storage = Arc::new(InMemoryProofStorage::new());
    ProofHelperEngineImpl::new(client.context(), storage)
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
    let file_hash_from_boc = engine.download_boc_and_calc_file_hash(1).await?;

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
    let proof_json = engine.query_mc_proof(1).await?;
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

    engine.add_file_hashes(&mut proofs).await?;

    for (seq_no, proof) in &proofs {
        let file_hash = engine.query_file_hash_from_next_block(*seq_no).await?.unwrap();
        assert_eq!(proof["file_hash"].as_str().unwrap(), file_hash);
    }

    Ok(())
}

#[tokio::test]
async fn mc_proofs_test() -> Result<()> {
    let engine = create_engine_mainnet();
    let trusted_id = resolve_initial_trusted_key_block(engine.context()).await?;

    let proof = BlockProof::from_value(&engine.query_mc_proof(100000).await?)?;
    proof.check_proof(&engine).await?;

    engine.storage().dump();

    assert_eq!(engine.storage().count(), 13);
    assert_eq!(engine.read_zs_right_bound().await?, 85049);

    let proof = BlockProof::from_value(&engine.query_mc_proof(trusted_id.seq_no + 100000).await?)?;
    proof.check_proof(&engine).await?;

    engine.storage().dump();

    assert_eq!(engine.storage().count(), 26);
    assert_eq!(engine.read_zs_right_bound().await?, 85049);
    assert_eq!(engine.read_trusted_block_right_bound(trusted_id.seq_no).await?, 11201794);

    Ok(())
}