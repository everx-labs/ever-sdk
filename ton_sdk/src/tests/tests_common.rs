/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use super::*;
use ed25519_dalek::{Keypair, SecretKey, PublicKey};
use futures::Stream;
use rand::rngs::OsRng;
use sha2::Sha512;
use std::str::FromStr;
use ton_block::{
    MsgAddressInt,
    TransactionProcessingStatus
};

const NODE_SE: bool = true;

const GIVER_ADDRESS_STR:  &str = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";

lazy_static! {
    static ref GIVER_ADDRESS: MsgAddressInt = MsgAddressInt::from_str(GIVER_ADDRESS_STR).unwrap();

    static ref WALLET_ADDRESS: MsgAddressInt = get_wallet_address(&WALLET_KEYS, 0);

    static ref WALLET_ADDRESS_BASE64: String = encode_base64(&WALLET_ADDRESS, false, false, false).unwrap();

    static ref WALLET_KEYS: Keypair = get_wallet_keys();
}

const DEFAULT_GIVER_KEYS: &str = r#"
{
    "secret": "2245e4f44af8af6bbd15c4a53eb67a8f211d541ddc7c197f74d7830dba6d27fe",
    "public": "d542f44146f169c6726c8cf70e4cbb3d33d8d842a4afd799ac122c5808d81ba3"
}"#;

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

fn get_wallet_address(key_pair: &Keypair, workchain_id: i32) -> MsgAddressInt {
    // create image to retrieve address
    let mut state_init = std::fs::File::open("src/tests/Wallet.tvc".to_owned()).expect("Unable to open contract code file");
    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let address = contract_image.msg_address(workchain_id);
    println!("Wallet address {} ({})", address, encode_base64(&address, false, false, false).unwrap());

    address
}

pub fn init_node_connection() {
    let config_json = if NODE_SE {
        r#"
        {
            "queries_config": {
                "queries_server": "http://0.0.0.0/graphql",
                "subscriptions_server": "ws://0.0.0.0/graphql"
            },
            "requests_config": {
                "requests_server": "http://0.0.0.0/topics/requests"
            }
        }"#
    } else {
        r#"
        {
            "queries_config": {
                "queries_server": "https://testnet.ton.dev/graphql",
                "subscriptions_server": "wss://testnet.ton.dev/graphql"
            },
            "requests_config": {
                "requests_server": "https://testnet.ton.dev/topics/requests"
            }
        }"#
    };

        
    init_json(config_json.into()).unwrap();
}

#[test]
#[ignore]
fn test_print_address() {
    get_wallet_address(&WALLET_KEYS, 0);
}

#[test]
fn test_generate_keypair_and_address() {
    // generate key pair
    let mut csprng = OsRng::new().unwrap();
    let key_pair = Keypair::generate::<Sha512, _>(&mut csprng);

    println!("Key pair: {}", hex::encode(&key_pair.to_bytes().to_vec()));

    get_wallet_address(&key_pair, 0);
}

#[test]
fn test_send_grams_from_giver() {
    init_node_connection();

    println!("Sending grams to {}", WALLET_ADDRESS.to_owned());

    call_contract(
        GIVER_ADDRESS.to_owned(),
        "sendGrams",
        json!({
            "dest": WALLET_ADDRESS.to_string(),
            "amount": 1_000_000_000_000u64
        }).to_string(),
        GIVER_ABI,
        None);
}

#[test]
#[ignore]
fn test_deploy_giver() {
    init_node_connection();

    deploy_contract_and_wait("Wallet.tvc", SIMPLE_WALLET_ABI, "{}", &WALLET_KEYS, 0);

    println!("Giver deployed. Address {}\n", WALLET_ADDRESS.to_string());
}

fn is_message_done(status: TransactionProcessingStatus) -> bool {
    (status == TransactionProcessingStatus::Preliminary) ||
    (status == TransactionProcessingStatus::Proposed) ||
    (status == TransactionProcessingStatus::Finalized)
}

fn wait_message_processed(changes_stream: Box<dyn Stream<Item = Transaction, Error = SdkError>>) -> Transaction {
    for state in changes_stream.wait() {
        match state {
            Ok(s) => {
                println!("{} : {:?}", s.id, s.status);
                if is_message_done(s.status) {
                    return s;
                }
            }
            Err(e) => panic!("error next state getting: {}", e)
        }
    }
    panic!("Error: no transaction id")
}

fn wait_message_processed_by_id(id: &MessageId)-> Transaction {
    wait_message_processed(Contract::subscribe_transaction_processing(id).unwrap())
}

fn check_giver() {
    let contract = Contract::load(&WALLET_ADDRESS)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    if let  Some(contract) = contract {
        if contract.balance_grams().unwrap() < 500_000_000 {
            panic!(format!(
                "Giver has no money. Send some grams to {} ({})",
                WALLET_ADDRESS.to_string(),
                WALLET_ADDRESS_BASE64.to_string()));
        }

        if contract.code.is_some() { return; }
    } else {
        panic!(format!(
            "Giver does not exist. Send some grams to {} ({})",
            WALLET_ADDRESS.to_string(),
            WALLET_ADDRESS_BASE64.to_string()));
    }

    println!("No giver. Deploy");

    test_deploy_giver();
}

pub fn get_grams_from_giver(address: MsgAddressInt) {
    println!("Account to take some grams {}", address);

    let transaction = if NODE_SE {
        if GIVER_ADDRESS.to_owned() == address {
            println!("Can not send to self");
            return;
        }

        call_contract(
            GIVER_ADDRESS.to_owned(),
            "sendGrams",
            json!({
                "dest": address.to_string(),
                "amount": 500_000_000u64
            }).to_string(),
            GIVER_ABI,
            None)
    } else {
        if WALLET_ADDRESS.to_owned() == address {
            println!("Can not send to self");
            return;
        }

        check_giver();

        call_contract(
            WALLET_ADDRESS.to_owned(),
            "sendTransaction",
            json!({
                "dest": format!("0x{:x}", address.get_address()),
                "value": 500_000_000u64,
                "bounce": false
            }).to_string(),
            SIMPLE_WALLET_ABI,
            Some(&WALLET_KEYS))
    };

    transaction.out_messages_id().iter().for_each(|msg_id| {
        wait_message_processed_by_id(&msg_id);
    });
}

pub fn deploy_contract_and_wait(code_file_name: &str, abi: &str, constructor_params: &str, key_pair: &Keypair, workchain_id: i32) -> MsgAddressInt {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/".to_owned() + code_file_name).expect("Unable to open contract code file");

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.msg_address(workchain_id);

    get_grams_from_giver(account_id.clone());

    // call deploy method
    let changes_stream = Contract::deploy_json("constructor".to_owned(), constructor_params.to_owned(), abi.to_owned(), contract_image, Some(key_pair), workchain_id)
        .expect("Error deploying contract");

    // wait transaction id in message-status
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let tr = wait_message_processed(changes_stream);

    if tr.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    account_id
}

pub fn call_contract(address: MsgAddressInt, func: &str, input: String, abi: &str, key_pair: Option<&Keypair>) -> Transaction {
    // call needed method
    let changes_stream = Contract::call_json(address, func.to_owned(), input, abi.to_owned(), key_pair)
        .expect("Error calling contract method");

    // wait transaction id in message-status
    let tr = wait_message_processed(changes_stream);

    // OR
    // wait message will done and find transaction with the message

    if tr.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    tr
}

#[allow(dead_code)]
pub fn call_contract_and_wait(address: MsgAddressInt, func: &str, input: String, abi: &str, key_pair: Option<&Keypair>)
    -> (String, Transaction)
{
    // call needed method
    let changes_stream =
        Contract::call_json(address, func.to_owned(), input, abi.to_owned(), key_pair)
            .expect("Error calling contract method");

    // wait transaction id in message-status
    let tr = wait_message_processed(changes_stream);

    // OR
    // wait message will done and find transaction with the message

    if tr.is_aborted() {
        panic!("transaction aborted!\n\n{:?}", tr)
    }

    let abi_contract = AbiContract::load(abi.as_bytes()).expect("Couldn't parse ABI");
    let abi_function = abi_contract.function(func).expect("Couldn't find function");

    // take external outbound message from the transaction
    let out_msg = tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| {
            let msg = msg.as_ref()
                .expect("error unwrap out message 1")
                .as_ref()
                    .expect("error unwrap out message 2");
            msg.msg_type() == MessageType::ExternalOutbound
            && msg.body().is_some()
            && abi_function.is_my_message(msg.body().expect("No body"), false).expect("error is_my_message")
        })
            .expect("erro unwrap out message 2")
            .expect("erro unwrap out message 3")
            .expect("erro unwrap out message 4");

    // take body from the message
    let responce = out_msg.body().expect("error unwrap out message body").into();

    //println!("response {}", responce);

    // decode the body by ABI
    let result = Contract::decode_function_response_json(abi.to_owned(), func.to_owned(), responce, false)
        .expect("Error decoding result");

    //println!("Contract call result: {}\n", result);

    (result, tr)

    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body
}

pub fn local_contract_call(address: MsgAddressInt, func: &str, input: &str, abi: &str, key_pair: Option<&Keypair>) -> String {

    let contract = Contract::load_wait_deployed(&address).expect("Error loading Contract");

    // call needed method
    let messages = contract.local_call_json(func.to_owned(), input.to_owned(), abi.to_owned(), key_pair)
        .expect("Error calling locally");

    for msg in messages {
        if msg.msg_type() == MessageType::ExternalOutbound {
            return Contract::decode_function_response_json(
                abi.to_owned(), func.to_owned(), msg.body().expect("Message has no body"), false)
                    .expect("Error decoding result");
        }
    }

    panic!("No output messages")
}

const GIVER_ABI: &str = r#"
{
	"ABI version": 1,
	"functions": [
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "sendGrams",
			"inputs": [
				{"name":"dest","type":"address"},
				{"name":"amount","type":"uint64"}
			],
			"outputs": [
			]
		}
	],
	"events": [
	],
	"data": [
	]
}"#;

const SIMPLE_WALLET_ABI: &str = r#"
{
    "ABI version": 1,
    "functions": [
        {
            "name": "constructor",
            "inputs": [
            ],
            "outputs": [
            ]
        },
        {
            "name": "sendTransaction",
            "inputs": [
                {"name":"dest","type":"address"},
                {"name":"value","type":"uint128"},
                {"name":"bounce","type":"bool"}
            ],
            "outputs": [
            ]
        }
    ],
    "events": [
    ],
    "data": [
        {"key":100,"name":"owner","type":"uint256"}
    ]
} "#;
