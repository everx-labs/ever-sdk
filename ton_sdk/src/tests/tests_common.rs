use super::*;
use ed25519_dalek::Keypair;
use futures::Stream;
use rand::rngs::OsRng;
use sha2::Sha512;
use std::str::FromStr;
use tvm::block::{
    MsgAddressInt,
    TransactionProcessingStatus
};

const NODE_SE: bool = true;

const GIVER_ADDRESS_STR:  &str = "0:a46af093b38fcae390e9af5104a93e22e82c29bcb35bf88160e4478417028884";
const WALLET_ADDRESS_STR: &str = "0:bba1ac23b010188089d62010ddb00d594c00f0e217794f3f2b53a81894ec7146";

lazy_static! {
    static ref GIVER_ADDRESS: MsgAddressInt = MsgAddressInt::from_str(GIVER_ADDRESS_STR).unwrap();

    static ref WALLET_ADDRESS: MsgAddressInt = MsgAddressInt::from_str(WALLET_ADDRESS_STR).unwrap();

    static ref WALLET_ADDRESS_STR_HEX: String = WALLET_ADDRESS.get_address().to_hex_string();

    static ref WALLET_KEYS: Keypair = Keypair::from_bytes(&hex::decode(
            "2245e4f44af8af6bbd15c4a53eb67a8f211d541ddc7c197f74d7830dba6d27fed542f44146f169c6726c8cf70e4cbb3d33d8d842a4afd799ac122c5808d81ba3"
        ).unwrap()).unwrap();
}

pub fn init_node_connection() {
    let config_json = if NODE_SE {
        r#"
        {
            "queries_config": {
                "queries_server": "http://192.168.99.100/graphql",
                "subscriptions_server": "ws://192.168.99.100/graphql"
            },
            "requests_config": {
                "requests_server": "http://192.168.99.100/topics/requests"
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

fn print_wallet_address(key_pair: &Keypair, workchain_id: i32) {
    // create image to retrieve address
    let mut state_init = std::fs::File::open("src/tests/Wallet.tvc".to_owned()).expect("Unable to open contract code file");
    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let address = contract_image.msg_address(workchain_id);

    println!("Base64 address for gram request: {}", encode_base64(&address, false, false, false).unwrap());
    println!("Hex address: {}", address);
}

#[test]
//#[ignore]
fn test_print_address() {
    print_wallet_address(&WALLET_KEYS, 0);
}

#[test]
fn test_generate_keypair_and_address() {
    // generate key pair
    let mut csprng = OsRng::new().unwrap();
    let key_pair = Keypair::generate::<Sha512, _>(&mut csprng);

    println!("Key pair: {}", hex::encode(&key_pair.to_bytes().to_vec()));

    print_wallet_address(&key_pair, 0);
}

#[test]
fn test_send_grams_from_giver() {
    init_node_connection();

    println!("Sending grams to {}", WALLET_ADDRESS.to_owned());

    call_contract(
        GIVER_ADDRESS.to_owned(),
        "sendGrams",
        json!({
            "dest": format!("0x{:x}", WALLET_ADDRESS.get_address()),
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

    println!("Giver deployed. Address {} ({:x})\n", WALLET_ADDRESS_STR, WALLET_ADDRESS.get_address());
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
    /*
    let contract = Contract::load(WALLET_ADDRESS.clone())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    if let  Some(contract) = contract {
        if contract.balance_grams().value() < &1_000_000_000u64.into() {
            panic!(format!(
                "Giver has no money. Send some grams to {} ({})",
                WALLET_ADDRESS_STR,
                WALLET_ADDRESS_STR_HEX.as_str()));
        }
    } else {
        panic!(format!(
            "Giver does not exist. Send some grams to {} ({})",
            WALLET_ADDRESS_STR,
            &WALLET_ADDRESS_STR_HEX.as_str()));
    }*/

    let result = queries_helper::query(
            "accounts",
            &json!({
                "id": {
                    "eq": WALLET_ADDRESS_STR_HEX.as_str()
                }
            }).to_string(),
            "storage {
                balance {
                    Grams
                }
                state {
                    ...on AccountStorageStateAccountActiveVariant {
                        AccountActive {
                            code
                        }
                    }
                }
            }",
            None,
            None)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    if result[0].is_null() {
        panic!(format!(
            "Giver does not exist. Send some grams to {} ({})",
            WALLET_ADDRESS_STR,
            &WALLET_ADDRESS_STR_HEX.as_str()));
    }

    if u64::from_str_radix(
            result[0]["storage"]["balance"]["Grams"].as_str().unwrap(),
            10)
        .unwrap() < 500_000_000u64
    {
        panic!(format!(
            "Giver has no money. Send some grams to {} ({})",
            WALLET_ADDRESS_STR,
            WALLET_ADDRESS_STR_HEX.as_str()));
    }

    if !result[0]["storage"]["state"]["AccountActive"].is_null()
    {
        return;
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
            "dest": format!("0x{:x}", address.get_address()),
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

    let contract = Contract::load(&address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

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
				{"name":"dest","type":"uint256"},
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
				{"name":"dest","type":"uint256"},
				{"name":"value","type":"uint128"},
				{"name":"bounce","type":"bool"}
			],
			"outputs": [
			]
		},
		{
			"name": "setSubscriptionAccount",
			"inputs": [
				{"name":"addr","type":"uint256"}
			],
			"outputs": [
			]
		},
		{
			"name": "getSubscriptionAccount",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint256"}
			]
		}
	],
	"events": [
	],
	"data": [
	]
} "#;
