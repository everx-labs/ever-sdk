use std::collections::HashMap;
use std::sync::Arc;
use std::env;
use std::str::FromStr;

use serde_json::json;
use ton_client::ClientContext;
use ton_client::net::{OrderBy, ParamsOfQueryCollection, query_collection, SortDirection};
use ton_client::proofs::{ParamsOfProofBlockData, proof_block_data};
use ton_types::{Result, UInt256};

fn with_project(endpoint: &str) -> String {
    let key = "EVERCLOUD_AUTH_PROJECT";
    match env::var(key) {
        Ok(project) => {
            if endpoint.ends_with('/') {
                format!("{}{}", endpoint, project)
            } else {
                format!("{}/{}", endpoint, project)
            }
        }
        Err(_) => endpoint.to_string(),
    }
}

async fn query_network_keyblocks(
    endpoint: String,
    zs_root_hash: UInt256,
    trusted_blocks: Option<Vec<(u32, [u8; 32])>>,
) -> Result<Vec<(u32, [u8; 32])>> {
    println!("*** [{}] ***", endpoint);
    let context = Arc::new(
        ClientContext::new(
            serde_json::from_value(json!({
                "network": {
                    "endpoints": [endpoint],
                }
            }))?
        )?
    );

    println!("Zerostate root_hash: {}", zs_root_hash.as_hex_string());

    let mut result = trusted_blocks.unwrap_or_default();
    let mut last_seq_no = match result.last() {
        Some((seq_no, _)) => *seq_no,
        None => 0,
    };
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
            println!("*** [{} done] ***", endpoint);
            return Ok(result);
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

    let networks = [
        ("mainnet.evercloud.dev", "58ffca1a178daff705de54216e5433c9bd2e7d850070d334d38997847ab9e845"),
        ("devnet.evercloud.dev", "cd81dae0c23d78e7c3eb5903f2a7bd98889991d36a26812a9163ca0f29c47093"),
    ];

    let mut trusted_key_blocks = match std::fs::read(&args[1]) {
        Ok(data) => {
            println!("Updating trusted blocks list in {}", &args[1]);
            bincode::deserialize(&data)?
        },
        Err(_) => {
            println!("Creating new trusted blocks list in {}", &args[1]);
            HashMap::new()
        }
    };


    for (network, zs_root_hash) in networks {
        let zs_root_hash = UInt256::from_str(zs_root_hash)?;
        let endpoint = with_project(network).to_owned();
        let value = query_network_keyblocks(
            endpoint,
            zs_root_hash.clone(),
            trusted_key_blocks.remove(&zs_root_hash.clone().inner()),
        ).await?;
        trusted_key_blocks.insert(zs_root_hash.inner(), value);
    }

    let data = bincode::serialize(&trusted_key_blocks)?;
    std::fs::write(&args[1], data)?;

    Ok(())
}
