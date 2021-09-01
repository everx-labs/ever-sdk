use ton_block::{BinTreeType, Block, BlockIdExt, Deserializable, InRefValue, MASTERCHAIN_ID, ShardHashes, ShardIdent};
use ton_types::Result;

use crate::proofs::BlockProof;
use crate::proofs::validators::{calc_subset_for_workchain, calc_workchain_id, calc_workchain_id_by_adnl_id};

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
fn test_check_shard_blocks_proof() -> Result<()> {
    for seq_no in 4377252..=4377282 {
        let block_proof = BlockProof::read_from_file(
            format!("src/proofs/tests/data/test_shard_block_proof/proof_{}", seq_no)
        )?;
        block_proof.check_proof_link()?;
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

    vset.list().iter().enumerate().for_each(|(i,descr)| {
        let real_id = calc_workchain_id(descr);
        println!("{}: pub_key: {} real_id: {}", i, hex::encode(descr.public_key.as_slice()), real_id);
    });

    for workchain_id in -1..=1 {
        println!("workchain_id: {}", workchain_id);
        let cc_config = config.catchain_config()?;
        let subset = calc_subset_for_workchain(&vset, config, &cc_config, ton_block::SHARD_FULL, workchain_id, cc_seqno, 0.into())?;
        assert_eq!(subset.0.len(), 7);
        subset.0.iter().enumerate().for_each(|(i,descr)| {
            let real_id = calc_workchain_id(descr);
            println!("{}: pub_key: {} real_id: {}", i, hex::encode(descr.public_key.as_slice()), real_id);
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

    let vset = config.validator_set()?;
    let election_id = vset.utime_since();
    println!("elections: {} total validators: {}", election_id, vset.list().len());

    let cc_seqno = block.read_info()?.gen_catchain_seqno();

    vset.list().iter().enumerate().for_each(|(i,descr)| {
        let id = calc_workchain_id(descr);
        println!("{}: pub_key: {} id: {}", i, hex::encode(descr.public_key.as_slice()), id);
    });

    let count = config.workchains()?.len()? as i32;
    for workchain_id in -1..count {
        let shard_ids = match workchain_id {
            MASTERCHAIN_ID => vec!(BlockIdExt::with_params(ShardIdent::masterchain(), 0, Default::default(), Default::default())),
            workchain_id => get_top_blocks(custom.shards(), &[workchain_id])?
        };
        for block_id in shard_ids {
            println!("{}", block_id.shard());
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
            subset.0.iter().enumerate().for_each(|(i,descr)| {
                let real_id = calc_workchain_id(descr);
                println!("{}: pub_key: {} real_id: {}", i, hex::encode(descr.public_key.as_slice()), real_id);
                assert_eq!(real_id, workchain_id);
            });
        }
    }

    Ok(())
}
