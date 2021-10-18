use std::collections::HashMap;
use std::sync::Arc;

use failure::bail;
use serde_json::json;
use ton_client::ClientContext;
use ton_client::net::{OrderBy, ParamsOfQueryCollection, query_collection, SortDirection};
use ton_client::proofs::{ParamsOfProofBlockData, proof_block_data};
use ton_types::{Result, UInt256};

async fn query_current_network_zs_hash(
    context: Arc<ClientContext>,
) -> Result<UInt256> {
    let mut blocks = query_collection(Arc::clone(&context), ParamsOfQueryCollection {
        collection: "blocks".to_string(),
        filter: Some(json!({
            "workchain_id": {
                "eq": -1
            },
            "seq_no": {
                "eq": 1
            },
        })),
        result: "id boc prev_ref{root_hash}".to_string(),
        limit: Some(1),
        ..Default::default()
    }).await?.result;

    if blocks.is_empty() {
        bail!("Unable to resolve zerostate's root hash: can't get masterchain block #1");
    }

    let block = blocks.remove(0);

    proof_block_data(Arc::clone(&context), ParamsOfProofBlockData {
        block: block.clone(),
    }).await?;

    let prev_ref = &block["prev_ref"];
    if prev_ref.is_null() {
        bail!("Unable to resolve zerostate's root hash: prev_ref of the block #1 is not set");
    }

    UInt256::from_str(
        prev_ref["root_hash"].as_str()
            .expect("Field `prev_ref.root_hash` must be a string")
    )
}

async fn query_network_keyblocks(server_address: &'static str) -> Result<(UInt256, Vec<(u32, [u8; 32])>)> {
    println!("*** [{}] ***", server_address);
    let context = Arc::new(
        ClientContext::new(
            serde_json::from_value(json!({
                "network": {
                    "server_address": server_address,
                }
            }))?
        )?
    );

    let zs_root_hash = query_current_network_zs_hash(Arc::clone(&context)).await?;

    println!("Zerostate root_hash: {}", zs_root_hash.as_hex_string());

    let mut result = Vec::new();
    let mut last_seq_no = 0;
    let mut last_gen_utime = 0;
    loop {
        let key_blocks = query_collection(
            Arc::clone(&context),
            ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                result: "id seq_no gen_utime boc".to_string(),
                filter: Some(json!({
                    "workchain_id": {
                        "eq": -1,
                    },
                    "key_block": {
                        "eq": true,
                    },
                    "seq_no": {
                        "gt": last_seq_no,
                    }
                })),
                order: Some(vec![
                    OrderBy {
                        path: "seq_no".to_string(),
                        direction: SortDirection::ASC,
                    },
                ]),
                ..Default::default()
            }
        ).await?.result;

        if key_blocks.is_empty() {
            println!("*** [{} done] ***", server_address);
            return Ok((zs_root_hash, result));
        }

        for key_block in key_blocks {
            let seq_no = key_block["seq_no"].as_u64()
                    .expect("Field `seq_no` must be an integer") as u32;
            print!("Proof for key_block #{}...", seq_no);
            proof_block_data(Arc::clone(&context), ParamsOfProofBlockData {
                block: key_block.clone(),
            }).await?;
            let root_hash = UInt256::from_str(key_block["id"].as_str()
                .expect("Field `id` must be a string"))?;
            println!(" OK. root_hash: {}", root_hash.as_hex_string());

            let gen_utime = key_block["gen_utime"].as_u64()
                .expect("Field `gen_utime` must be an integer");
            if seq_no != last_seq_no {
                last_seq_no = seq_no;
                last_gen_utime = gen_utime;
                result.push((seq_no, root_hash.inner()));
            } else if gen_utime > last_gen_utime {
                last_gen_utime = gen_utime;
                let last_index = result.len() - 1;
                result[last_index].1 = root_hash.inner();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file name with path to save trusted key blocks>", args[0]);
        std::process::exit(1);
    }

    let mut trusted_key_blocks = HashMap::new();

    let networks = [
        "main.ton.dev",
        "net.ton.dev",
    ];

    for network in networks {
        let (key, value) = query_network_keyblocks(network).await?;
        trusted_key_blocks.insert(key.inner(), value);
    }

    let data = bincode::serialize(&trusted_key_blocks)?;
    std::fs::write(&args[1], data)?;

    Ok(())
}
