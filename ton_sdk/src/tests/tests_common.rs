/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use super::*;
use ed25519_dalek::{Keypair, SecretKey, PublicKey};
use std::str::FromStr;
use ton_block::MsgAddressInt;

const NODE_SE: bool = true;

const GIVER_ADDRESS_STR: &str = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";
const WALLET_ADDRESS_STR: &str = "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13";

pub const CONTRACTS_PATH: &str = "src/tests/contracts/";

lazy_static::lazy_static! {
    static ref GIVER_ADDRESS: MsgAddressInt = MsgAddressInt::from_str(GIVER_ADDRESS_STR).unwrap();
    static ref WALLET_ADDRESS: MsgAddressInt = MsgAddressInt::from_str(WALLET_ADDRESS_STR).unwrap();
    static ref WALLET_KEYS: Keypair = get_wallet_keys();

	pub static ref SUBSCRIBE_CONTRACT_ABI: String = std::fs::read_to_string(CONTRACTS_PATH.to_owned() + "Subscription.abi.json").unwrap();
	pub static ref PIGGY_BANK_CONTRACT_ABI: String = std::fs::read_to_string(CONTRACTS_PATH.to_owned() + "Piggy.abi.json").unwrap();
    pub static ref WALLET_ABI: String = std::fs::read_to_string(CONTRACTS_PATH.to_owned() + "LimitWallet.abi.json").unwrap();
    pub static ref SIMPLE_WALLET_ABI: String = std::fs::read_to_string(CONTRACTS_PATH.to_owned() + "Wallet.abi.json").unwrap();
    pub static ref GIVER_ABI: String = std::fs::read_to_string(CONTRACTS_PATH.to_owned() + "Giver.abi.json").unwrap();
    pub static ref PROFESSOR_ABI: String = std::fs::read_to_string(CONTRACTS_PATH.to_owned() + "Professor.abi.json").unwrap();

    pub static ref SUBSCRIBE_CONTRACT_IMAGE: Vec<u8> = std::fs::read(CONTRACTS_PATH.to_owned() + "Subscription.tvc").unwrap();
	pub static ref PIGGY_BANK_CONTRACT_IMAGE: Vec<u8> = std::fs::read(CONTRACTS_PATH.to_owned() + "Piggy.tvc").unwrap();
    pub static ref WALLET_IMAGE: Vec<u8> = std::fs::read(CONTRACTS_PATH.to_owned() + "LimitWallet.tvc").unwrap();
    pub static ref SIMPLE_WALLET_IMAGE: Vec<u8> = std::fs::read(CONTRACTS_PATH.to_owned() + "Wallet.tvc").unwrap();
    pub static ref PROFESSOR_IMAGE: Vec<u8> = std::fs::read(CONTRACTS_PATH.to_owned() + "Professor.tvc").unwrap();
}

const DEFAULT_GIVER_KEYS: &str = r#"
{
    "secret": "2245e4f44af8af6bbd15c4a53eb67a8f211d541ddc7c197f74d7830dba6d27fe",
    "public": "d542f44146f169c6726c8cf70e4cbb3d33d8d842a4afd799ac122c5808d81ba3"
}"#;

pub fn get_config() -> serde_json::Value {
    let network = std::env::var("TON_NETWORK_ADDRESS")
        .unwrap_or(if NODE_SE {
            "http://localhost:80"
        } else {
            "net.ton.dev"
        }.to_owned());
    json!({
        "base_url": network
    })
}

pub async fn init_node_connection() -> NodeClient {
    let config_json = get_config().to_string();

    init_json(&config_json).await.unwrap()
}

fn get_wallet_keys() -> Keypair {
    let mut keys_file = dirs::home_dir().unwrap();
    keys_file.push("giverKeys.json");
    let keys = std::fs::read_to_string(keys_file).unwrap_or(DEFAULT_GIVER_KEYS.to_owned());

    let keys: serde_json::Value = serde_json::from_str(&keys).unwrap();

    println!("Using keys\n{}", keys);

    Keypair {
        secret: SecretKey::from_bytes(&hex::decode(keys["secret"].as_str().unwrap()).unwrap()).unwrap(),
        public: PublicKey::from_bytes(&hex::decode(keys["public"].as_str().unwrap()).unwrap()).unwrap(),
    }
}

#[tokio::main]
#[ignore]
#[test]
async fn test_send_grams_from_giver() {
    let client = init_node_connection().await;

    println!("Sending grams to {}", WALLET_ADDRESS.to_owned());

    call_contract(
        &client,
        GIVER_ADDRESS.to_owned(),
        "sendGrams",
        json!({
            "dest": WALLET_ADDRESS.to_string(),
            "amount": 1_000_000_000_000u64
        }).to_string(),
        &GIVER_ABI,
        None).await;
}

#[tokio::main]
#[ignore]
#[test]
async fn test_deploy_giver() {
    let client = init_node_connection().await;

    deploy_contract_and_wait(&client, &SIMPLE_WALLET_IMAGE, &SIMPLE_WALLET_ABI, "{}", &WALLET_KEYS, 0).await;

    println!("Giver deployed. Address {}\n", WALLET_ADDRESS.to_string());
}

async fn check_giver(client: &NodeClient) {
    let contract = Contract::load(client, &WALLET_ADDRESS)
        .await
        .expect("Error unwrap result while loading Contract");

    if let  Some(contract) = contract {
        if contract.balance_grams() < 500_000_000 {
            panic!(format!(
                "Giver has no money. Send some grams to {}",
                WALLET_ADDRESS.to_string()));
        }

        if contract.code_hash.is_some() || contract.get_code().is_some() { return; }
    } else {
        panic!(format!(
            "Giver does not exist. Send some grams to {}",
            WALLET_ADDRESS.to_string()));
    }

    println!("No giver. Deploy");

    test_deploy_giver();
}

pub async fn get_grams_from_giver(client: &NodeClient, address: MsgAddressInt) {
    println!("Account to take some grams {}", address);

    let transaction = if NODE_SE {
        if GIVER_ADDRESS.to_owned() == address {
            println!("Can not send to self");
            return;
        }

        call_contract(
            client,
            GIVER_ADDRESS.to_owned(),
            "sendGrams",
            json!({
                "dest": address.to_string(),
                "amount": 500_000_000u64
            }).to_string(),
            &GIVER_ABI,
            None).await
    } else {
        if WALLET_ADDRESS.to_owned() == address {
            println!("Can not send to self");
            return;
        }

        check_giver(client).await;

        call_contract(
            client,
            WALLET_ADDRESS.to_owned(),
            "sendTransaction",
            json!({
                "dest": address.to_string(),
                "value": 200_000_000u64,
                "bounce": false
            }).to_string(),
            &SIMPLE_WALLET_ABI,
            Some(&WALLET_KEYS)).await
    };

    for msg_id in transaction.out_messages_id() {
        client.wait_for(
            crate::types::TRANSACTIONS_TABLE_NAME,
            &json!({
                "in_msg": { "eq": msg_id.to_string() },
            }).to_string(),
            "id",
            None)
            .await
            .expect("Error waiting giver message processing");
    }
}

pub async fn deploy_contract_and_wait(
    client: &NodeClient,
    mut contract_image: &[u8],
    abi: &str,
    constructor_params: &str,
    key_pair: &Keypair,
    workchain_id: i32,
) -> MsgAddressInt {
    let contract_image = ContractImage::from_state_init_and_key(&mut contract_image, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.msg_address(workchain_id);

    get_grams_from_giver(client, account_id.clone()).await;

    let now = std::time::Instant::now();

    // call deploy method
    let msg = Contract::construct_deploy_message_json(
        FunctionCallSet {
            func: "constructor".to_owned(),
            header: None,
            input: constructor_params.to_owned(),
            abi: abi.to_owned(),
        },
        contract_image,
        Some(key_pair),
        workchain_id,
        Some(client.timeouts()),
        None,
    ).unwrap();

    let tr = Contract::process_message(&client, &msg, false).await;

    let t = now.elapsed();
    println!("Deploy time {}.{:03} ", t.as_secs(), t.subsec_millis());
    println!("now {}", Contract::now());

    let tr = tr.expect("Error deploying contract");

    println!("Transaction now {}", tr.parsed.now);
    if tr.parsed.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    account_id
}

pub async fn call_contract(
    client: &NodeClient,
    address: MsgAddressInt,
    func: &str,
    input: String,
    abi: &str,
    key_pair: Option<&Keypair>,
) -> Transaction {
    let now = std::time::Instant::now();
    // call needed method
    let msg = Contract::construct_call_message_json(
        address,
        FunctionCallSet {
            func: func.to_owned(),
            header: None,
            input,
            abi: abi.to_owned(),
        },
        false,
        key_pair,
        Some(client.timeouts()),
        None,
    ).unwrap();

    let tr = Contract::process_message(&client, &msg, false).await;

    let t = now.elapsed();
    println!("Call time {}.{:03} ", t.as_secs(), t.subsec_millis());
    println!("now {}", Contract::now());

    let tr = tr.expect("Error calling contract method");

    println!("Transaction now {}", tr.parsed.now);
    if tr.parsed.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    tr.parsed
}

#[allow(dead_code)]
pub async fn call_contract_and_wait(
    client: &NodeClient,
    address: MsgAddressInt,
    func: &str,
    input: String,
    abi: &str,
    key_pair: Option<&Keypair>,
) -> String {
    let now = std::time::Instant::now();
    // call needed method
    let msg = Contract::construct_call_message_json(
        address,
        FunctionCallSet {
            func: func.to_owned(),
            header: None,
            input,
            abi: abi.to_owned(),
        },
        false,
        key_pair,
        Some(client.timeouts()),
        None,
    ).unwrap();

    let tr = Contract::process_message(&client, &msg, false)
        .await;

    let t = now.elapsed();
    println!("Call time {}.{:03} ", t.as_secs(), t.subsec_millis());
    println!("now {}", Contract::now());

    let tr = tr.expect("Error calling contract method");

    if tr.parsed.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    let abi_contract = AbiContract::load(abi.as_bytes()).expect("Couldn't parse ABI");
    let abi_function = abi_contract.function(func).expect("Couldn't find function");

    let mut result = "{}".to_owned();

    // decode the body by ABI
    for msg in tr.parsed.out_messages {
        if msg.msg_type() == MessageType::ExternalOutbound &&
            abi_function.is_my_output_message(msg.body().unwrap(), false).unwrap()
        {
            result = Contract::decode_function_response_json(
                abi.to_string(), func.to_string(), msg.body().unwrap(), false).unwrap();
            //println!("Contract call result: {}\n", output);
            break;
        }
    }

    result
}

pub async fn contract_call_local(
    client: &NodeClient,
    address: MsgAddressInt,
    func: &str,
    input: &str,
    abi: &str,
    key_pair: Option<&Keypair>,
) -> String {
    let contract = Contract::load_wait(client, &address, true, None)
        .await
        .expect("Error unwrap result while loading Contract");

    // call needed method
    let messages = contract.local_call_tvm_json(func.to_owned(), None, input.to_owned(), abi.to_owned(), key_pair)
        .expect("Error calling locally");

    for msg in messages {
        if msg.msg_type() == MessageType::ExternalOutbound {
            return Contract::decode_function_response_json(
                abi.to_owned(), func.to_owned(), msg.body().expect("Message has no body"), false)
                .expect("Error decoding result");
        }
    }

    "{}".to_owned()
}
