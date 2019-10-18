extern crate ton_sdk;
extern crate hex;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate num_bigint;
extern crate clap;

use clap::{Arg, App};
use ed25519_dalek::Keypair;
use futures::Stream;
use num_traits::cast::ToPrimitive;
use rand::{thread_rng, Rng};
use sha2::Sha512;
use std::str::FromStr;
use ton_sdk::*;
use tvm::block::{
    Message, MessageId, MsgAddressExt, MsgAddressInt, InternalMessageHeader, Grams, 
    ExternalInboundMessageHeader, CurrencyCollection, Serializable, 
    TransactionProcessingStatus, TransactionId
};
use tvm::stack::{BuilderData, IBitstring};
use tvm::types::{AccountId};

const WALLET_ABI: &str = r#"{
    "ABI version" : 0,
    "functions" :	[{
            "inputs": [
                {"name": "recipient", "type": "fixedbytes32"},
                {"name": "value", "type": "gram"}
            ],
            "name": "sendTransaction",
            "outputs": [
                {"name": "transaction", "type": "uint64"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "type", "type": "uint8"},
                {"name": "value", "type": "gram"},
                {"name": "meta", "type": "bytes"}
            ],
            "name": "createLimit",
            "outputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "value", "type": "gram"},
                {"name": "meta", "type": "bytes"}
            ],
            "name": "changeLimitById",
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "removeLimit",
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "getLimitById",
            "outputs": [
                {
                    "name": "limitInfo",
                    "type": "tuple",
                    "components": [
                        {"name": "value", "type": "gram"},
                        {"name": "type", "type": "uint8"},
                        {"name": "meta", "type": "bytes"}
                        ]
                },
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [],
            "name": "getLimits",
            "outputs": [
                {"name": "list", "type": "uint8[]"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [],
            "name": "getVersion",
            "outputs": [
                {
                    "name": "version",
                    "type": "tuple",
                    "components": [
                        {"name": "major", "type": "uint16"},
                        {"name": "minor", "type": "uint16"}
                    ]
                },
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [],
            "name": "getBalance",
            "outputs": [{"name": "balance", "type": "uint64"}]
        }, {
            "inputs": [],
            "name": "constructor",
            "outputs": []
        }, {
            "inputs": [{"name": "address", "type": "fixedbytes32" }],
            "name": "setSubscriptionAccount",
            "outputs": []
        }, {
            "inputs": [],
            "name": "getSubscriptionAccount",
            "outputs": [{"name": "address", "type": "fixedbytes32" }]
        }
    ]
}
"#;

fn str_grams_to_nanorams(grams: &str) -> String {
    let grams = f64::from_str(grams).expect("Couldn't parse number");
    let nanograms = grams * 1000000000 as f64;
    format!("{}", nanograms as u64)
}

fn is_message_done(status: TransactionProcessingStatus) -> bool {
    (status == TransactionProcessingStatus::Preliminary) ||
    (status == TransactionProcessingStatus::Proposed) ||
    (status == TransactionProcessingStatus::Finalized)
}

fn wait_message_processed(
    changes_stream: Box<dyn Stream<Item = ContractCallState, Error = ton_sdk::SdkError>>
    ) -> TransactionId
{
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("{} : {:?}", s.id.to_hex_string(), s.status);
            if is_message_done(s.status) {
                tr_id = Some(s.id.clone());
                break;
            }
        }
    }
    tr_id.expect("Error: no transaction id")
}

fn wait_message_processed_by_id(message_id: MessageId)-> TransactionId {
    wait_message_processed(Contract::subscribe_updates(message_id.clone()).unwrap())
}

// Create message "from wallet" to transfer some funds 
// from one account to another
pub fn create_external_transfer_funds_message(src: MsgAddressInt, dst: MsgAddressInt, value: u128) -> Message {
    
    let mut rng = thread_rng();    
    let mut builder = BuilderData::new();
    builder.append_u64(rng.gen::<u64>()).unwrap();    
    let mut msg = Message::with_ext_in_header(
        ExternalInboundMessageHeader {
            src: MsgAddressExt::with_extern(builder.into()).unwrap(),
            dst: src.clone(),
            import_fee: Grams::default(),
        }
    );

    let mut balance = CurrencyCollection::default();
    balance.grams = Grams(value.into());

    let int_msg_hdr = InternalMessageHeader::with_addresses(src, dst, balance);

    *msg.body_mut() = Some(int_msg_hdr.write_to_new_cell().unwrap().into());

    msg
}

fn deploy_contract_and_wait(code_file_name: &str, abi: &str, constructor_params: &str, key_pair: &Keypair) -> MsgAddressInt {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open(code_file_name).expect("Unable to open contract code file");

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id(0);

    // before deploying contract need to transfer some funds to its address
    //println!("Account ID to take some grams {}\n", account_id.to_hex_string());
    let address = MsgAddressInt::with_standart(None, 0, AccountId::from([0; 32])).unwrap();
    let msg = create_external_transfer_funds_message(address, account_id.clone(), 100000000000);
    let changes_stream = Contract::send_message(msg).expect("Error calling contract method");

    // wait transaction id in message-status 
    let tr_id = wait_message_processed(changes_stream);
    
    let tr = Transaction::load(tr_id)
        .expect("Error load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction");

    //println!("transaction:\n\n{}", serde_json::to_string_pretty(tr.tr()).unwrap());

    if tr.tr().is_aborted() {
        panic!("transaction aborted!\n\n{}", serde_json::to_string_pretty(tr.tr()).unwrap())
    }

    tr.out_messages_id().iter().for_each(|msg_id| {
        wait_message_processed_by_id(msg_id.clone());
    });

    // call deploy method
    let changes_stream = Contract::deploy_json("constructor".to_owned(), constructor_params.to_owned(), abi.to_owned(), contract_image, Some(key_pair))
        .expect("Error deploying contract");

    // wait transaction id in message-status 
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let tr_id = wait_message_processed(changes_stream);

    let tr = Transaction::load(tr_id)
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap returned Transaction");

    if tr.tr().is_aborted() {
        panic!("transaction aborted!\n\n{}", serde_json::to_string_pretty(tr.tr()).unwrap())
    }

    account_id
}

fn call_contract_and_wait(address: MsgAddressInt, func: &str, input: String, abi: &str, key_pair: Option<&Keypair>) -> String {
    // call needed method
    let changes_stream = 
        Contract::call_json(address, func.to_owned(), input, abi.to_owned(), key_pair)
            .expect("Error calling contract method");

    // wait transaction id in message-status 
    let tr_id = wait_message_processed(changes_stream);

    // OR 
    // wait message will done and find transaction with the message

    // load transaction object
    let tr = Transaction::load(tr_id)
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction")
        .expect("Error unwrap got Transaction");

    // take external outbound message from the transaction
    let out_msg = tr.load_out_messages()
        .expect("Error calling load out messages")
        .wait()
        .find(|msg| {
            msg.as_ref()
                .expect("error unwrap out message 1")
                .as_ref()
                    .expect("error unwrap out message 2")
                    .msg_type() == MessageType::ExternalOutbound
        })
            .expect("erro unwrap out message 2")
            .expect("erro unwrap out message 3")
            .expect("erro unwrap out message 4");

    // take body from the message
    let responce = out_msg.body().expect("error unwrap out message body");

    // decode the body by ABI
    let result = Contract::decode_function_response_json(abi.to_owned(), func.to_owned(), responce)
        .expect("Error decoding result");

    //println!("Contract call result: {}\n", result);

    result

    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body
}

fn call_create(current_address: &mut Option<MsgAddressInt>) {
    println!("Creating new wallet account");

    // generate key pair
    let mut csprng = rand::rngs::OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);
   
    // deploy wallet
    let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);
    let str_address = wallet_address.get_address().to_hex_string();

    println!("Acoount created. Address {}", str_address);

    std::fs::write("last_address", wallet_address.to_string()).expect("Couldn't save wallet address");
    std::fs::write(str_address, &keypair.to_bytes().to_vec()).expect("Couldn't save wallet key pair");

    *current_address = Some(wallet_address);
}

fn call_get_balance(current_address: &Option<MsgAddressInt>, params: &[&str]) {
    let address = if params.len() > 0 {
        MsgAddressInt::from_str(params[0]).unwrap()
    } else if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let contract = Contract::load(&address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    let nanogram_balance = contract.balance_grams();
    let nanogram_balance = nanogram_balance.unwrap().value().to_u128().expect("error cast grams to u128");
    let gram_balance = nanogram_balance as f64 / 1000000000f64;

    println!("Account balance {}", gram_balance);
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct SendTransactionAnswer {
    transaction: String,
    error: String
}

fn read_keypair(address: &MsgAddressInt) -> Vec<u8> {
    let file_name = address.get_address().to_hex_string();
    std::fs::read(file_name).expect("Couldn't read key pair")
}

fn call_send_transaction(current_address: &Option<MsgAddressInt>, params: &[&str]) {
    if params.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    println!("Sending {} grams to {}", params[1], params[0]);

    let nanogram_value = params[1].to_owned() + "000000000";

    let str_params = format!("{{ \"recipient\" : \"x{}\", \"value\": \"{}\" }}", params[0], nanogram_value);

    let pair = read_keypair(&address);
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "sendTransaction", str_params, WALLET_ABI, Some(&pair));


    let answer: SendTransactionAnswer = serde_json::from_str(&answer).unwrap();

    let transaction = u64::from_str_radix(&answer.transaction[2..], 16).expect("Couldn't parse transaction number");

    println!("Transaction ID {}", transaction);
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct CreateLimitAnswer {
    limitId: String,
    error: String
}

fn call_create_limit(current_address: &Option<MsgAddressInt>, params: &[&str]) {
    if params.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let limit_type = u8::from_str_radix(&params[0], 10).unwrap();

    let meta = if limit_type == 1 {
        if params.len() < 3
        {
            println!("Not enough parameters");
            return;
        }

        let period = u8::from_str_radix(&params[2], 10).unwrap();
        hex::encode(&[period])
    } else {
        String::new()
    };

    let nanogram_value = str_grams_to_nanorams(params[1]);

    let str_params = format!(r#"{{ "type" : "{}", "value": "{}", "meta": "x{}" }}"#, params[0], nanogram_value, meta);

    let pair = read_keypair(&address);
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "createLimit", str_params, WALLET_ABI, Some(&pair));


    let answer: CreateLimitAnswer = serde_json::from_str(&answer).unwrap();

    let limit_id = u8::from_str_radix(&answer.limitId[2..], 16).expect("Couldn't parse limit ID");

    println!("Limit ID {}", limit_id);

}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ChangeLimitAnswer {
    error: String
}

fn call_change_limit(current_address: &Option<MsgAddressInt>, params: &[&str]) {
    if params.len() < 2 {
        println!("Not enough parameters");
        return;
    }

    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let meta = if params.len() > 2 {
        let period = u8::from_str_radix(&params[2], 10).unwrap();
        hex::encode(&[period])
    } else {
        String::new()
    };

    let nanogram_value = str_grams_to_nanorams(params[1]);

    let str_params = format!(r#"{{ "limitId" : "{}", "value": "{}", "meta": "x{}" }}"#, params[0], nanogram_value, meta);

    let pair = read_keypair(&address);
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "changeLimitById", str_params, WALLET_ABI, Some(&pair));


    let _answer: ChangeLimitAnswer = serde_json::from_str(&answer).unwrap();

    println!("Limit changed successfully");
}

fn call_remove_limit(current_address: &Option<MsgAddressInt>, params: &[&str]) {
    if params.len() < 1 {
        println!("Not enough parameters");
        return;
    }

    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let str_params = format!(r#"{{ "limitId" : "{}" }}"#, params[0]);

    let pair = read_keypair(&address);
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "removeLimit", str_params, WALLET_ABI, Some(&pair));


    let _answer: ChangeLimitAnswer = serde_json::from_str(&answer).unwrap();

    println!("Limit removed successfully");
}

#[derive(Deserialize)]
struct LimitInfo {
    value: String,
    #[serde(rename="type")]
    kind: String,
    meta: String
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Deserialize)]
struct GetLimitByIdAnswer {
    limitInfo: LimitInfo,
    error: String
}

fn call_get_limit_by_id(current_address: &Option<MsgAddressInt>, params: &[&str]) {
    if params.len() < 1 {
        println!("Not enough parameters");
        return;
    }

    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let str_params = format!(r#"{{ "limitId" : "{}" }}"#, params[0]);

    let answer = call_contract_and_wait(address, "getLimitById", str_params, WALLET_ABI, None);


    let answer: GetLimitByIdAnswer = serde_json::from_str(&answer).unwrap();

    println!("\nLimit info:");
    println!("ID - {}", params[0]);
    let value = u64::from_str_radix(&answer.limitInfo.value[2..], 16).unwrap();
    let value = value as f64 / 1000000000 as f64;
    println!("Value - {}", value);
    
    if answer.limitInfo.kind == "0x0" {
        println!("Type - Single operation limit");
    } else {
        println!("Type - Arbitrary limit");
        let period = u64::from_str_radix(&answer.limitInfo.meta[2..], 16).unwrap();
        println!("Period - {} days", period);
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GetLimitsAnswer {
    list: Vec<String>,
    error: String
}

fn call_get_limits(current_address: &Option<MsgAddressInt>) {
    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let str_params = "{}".to_owned();

    let answer = call_contract_and_wait(address, "getLimits", str_params, WALLET_ABI, None);


    let answer: GetLimitsAnswer = serde_json::from_str(&answer).unwrap();

    println!("Limits count {}", answer.list.len());

    for limit in answer.list {
        call_get_limit_by_id(current_address, &[&limit]);
    };
}

#[derive(Deserialize)]
struct Version {
    major: String,
    minor: String
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GetVersionAnswer {
    version:Version,
    error: String
}

fn call_get_version(current_address: &Option<MsgAddressInt>) {
    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let str_params = "{}".to_owned();

    let answer = call_contract_and_wait(address, "getVersion", str_params, WALLET_ABI, None);


    let answer: GetVersionAnswer = serde_json::from_str(&answer).unwrap();

    let major = u16::from_str_radix(&answer.version.major[2..], 16).unwrap();
    let minor = u16::from_str_radix(&answer.version.minor[2..], 16).unwrap();

    println!("Wallet version {}.{}", major, minor);
}

fn set_address(current_address: &mut Option<MsgAddressInt>, params: &[&str]) {
    if params.len() < 1 {
        println!("Not enough parameters");
        return;
    }

    if let Ok(address) = MsgAddressInt::from_str(params[0]) {
        if std::fs::read(address.get_address().to_hex_string()).is_err() {
            println!("No key pair for this address. Can't work");
            return;
        }

        std::fs::write("last_address", params[0]).expect("Couldn't save wallet address");

        println!("New wallet address {}", params[0]);

        *current_address = Some(address);
    } else {
        println!("Couldn't parse address");
    }
}

extern crate reqwest;
extern crate base64;

//use kafka::producer::{Producer, Record, RequiredAcks};
use ton_sdk::NodeClientConfig;
use self::reqwest::Client;
use self::reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::io::Read;

#[derive(Clone, Serialize, Deserialize)]
struct AccountData {
    id: AccountId,
    keypair: Vec<u8>,
}

pub fn send_message(client: &Client, server: &str, key: &[u8], value: &[u8]) {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let key_encoded = base64::encode(key);
    let value_encoded = base64::encode(value);
    let body = json!({
        "records": [{ "key": key_encoded, "value": value_encoded }]
    });

    let result = client.post(server)
        .headers(headers)
        .body(body.to_string())
        .send();
    match result {
        Ok(result) => {
            if !result.status().is_success() {
                let bytes: Vec<u8> = result.bytes().map(|b| if let Ok(b) = b { b } else { 0 }).collect();
                let text = match String::from_utf8(bytes.clone()) {
                    Ok(text) => text,
                    Err(_) => hex::encode(bytes)
                };
                panic!(format!("Request failed: {}", text));
            }
        }
        Err(err) => panic!(format!("Can not send request: {}", err))
    }
}

fn create_cycle_test_thread(config: String, accounts: Vec<AccountData>, timeout: u64, msg_count: u32, thread_number: usize) -> std::thread::JoinHandle<()> {
    // a little delay for Kafka producer creation
    std::thread::sleep(std::time::Duration::from_millis(10));

    std::thread::spawn(move || {
        let acc_count = accounts.len() as u32;

        let accounts: Vec<(AccountId, Keypair)> = accounts.iter().cloned().map(|data| (data.id, Keypair::from_bytes(&data.keypair[..]).unwrap())).collect();

        let config: NodeClientConfig = serde_json::from_str(&config).expect("Couldn't parse config");

       /* let mut prod = Producer::from_hosts(config.)
                .with_ack_timeout(Duration::from_millis(config.kafka_config.ack_timeout))
                .with_required_acks(RequiredAcks::One)
                .create()
                .expect("Couldn't connect to Kafka");*/

        let client = Client::new();

        println!("Thread {}. Transfer cycle...", thread_number);
        let now = std::time::SystemTime::now();

        let mut sleeps = 0;
        for i in 0..msg_count {

            let (address_from, keypair) = &accounts[(i % acc_count) as usize];
            let (address_to, _) = &accounts[((i + 1) % acc_count) as usize];
            let value = 1000000;

            //println!("Sending {} nanograms from {} to {}", value, address_from.to_hex_string(), address_to.to_hex_string());

            let str_params = format!("{{ \"recipient\" : \"x{}\", \"value\": \"{}\" }}", address_to.to_hex_string(), value);

            let (msg, id) = Contract::construct_call_message_json(
                MsgAddressInt::with_standart(None, 0, address_from.clone()).unwrap(),
                "sendTransaction".to_owned(),
                str_params.to_owned(),
                WALLET_ABI.to_owned(),
                Some(&keypair)
            ).expect("Error generating message");

            //println!("msg id {:?}", id);

            //prod.send(&Record::from_key_value(&config.kafka_config.topic, &id.data.as_slice()[..], msg)).expect("Couldn't send message");

            send_message(&client, &config.requests_config.requests_server, &id.data.as_slice()[..], &msg);

           // Contract::send_serialized_message(id, &msg).expect("Error sending message");

            std::thread::sleep(std::time::Duration::from_millis(timeout));
            sleeps += timeout;

        }

        println!("Thread {}. Finished in {} ms", thread_number, now.elapsed().unwrap().as_millis() - sleeps as u128);
    })
}

fn cycle_test(config: String, params: &[&str]) {
    if params.len() < 4 {
        println!("Not enough parameters");
        return;
    }

    let acc_count = match u32::from_str_radix(params[0], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing accounts count");
            return;
        }
    };

    let timeout = match u64::from_str_radix(params[1], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing timeout");
            return;
        }
    };

    let msg_count = match u32::from_str_radix(params[2], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing messages count");
            return;
        }
    };

    let thread_count = match usize::from_str_radix(params[3], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing thread count");
            return;
        }
    };

    println!("Processing {} accounts in {} messages in {} threads", acc_count, msg_count, thread_count);

    println!("Accounts creating...");
    let mut csprng = rand::rngs::OsRng::new().unwrap();
    let mut accounts = Vec::new();
    for _ in 0..acc_count {
        // generate key pair
        let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

        // deploy wallet
        let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);

        accounts.push(AccountData { 
                id: wallet_address.get_address(),
                keypair: keypair.to_bytes().to_vec() 
            });
    }

    let mut threads = vec![];

    for i in 0..thread_count {
        threads.push(create_cycle_test_thread(config.clone(), accounts.clone(), timeout, msg_count, i));
    }

    for thread in threads {
        let _ = thread.join();
    }

    println!("The end");
}

fn cycle_test_init(params: &[&str]) {
    if params.len() < 1 {
        println!("Not enough parameters");
        return;
    }

    let acc_count = match u32::from_str_radix(params[0], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing accounts count");
            return;
        }
    };

    println!("Creating {} accounts", acc_count);
    let mut csprng = rand::rngs::OsRng::new().unwrap();

    let mut vec = Vec::new();

    for _ in 0..acc_count {
        // generate key pair
        let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

        // deploy wallet
        let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);

        vec.push(AccountData { 
                id: wallet_address.get_address(),
                keypair: keypair.to_bytes().to_vec() 
            });

        //vec.append(&mut keypair.to_bytes().to_vec());
        //vec.append(&mut wallet_address.as_slice().to_vec());
    }

    let data = serde_json::to_string_pretty(&vec).expect("Couldn't serialize accounts data");

    std::fs::write("test_accounts", data).expect("Couldn't write accounts data");

    println!("\nAccounts succesfully created");
}


fn cycle_test_run(config: String, params: &[&str]) {
    if params.len() < 3 {
        println!("Not enough parameters");
        return;
    }

    let timeout = match u64::from_str_radix(params[0], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing timeout");
            return;
        }
    };

    let msg_count = match u32::from_str_radix(params[1], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing messages count");
            return;
        }
    };

    let thread_count = match usize::from_str_radix(params[2], 10) {
        Ok(n) => n,
        _ => {
            println!("error parsing thread count");
            return;
        }
    };

    let vec = std::fs::read_to_string("test_accounts").expect("Couldn't read accounts data");

    let accounts: Vec<AccountData> = serde_json::from_str(&vec).expect("Couldn't deserialize accounts data");
    let acc_count = accounts.len();

    println!("Processing {} accounts in {} messages in {} threads", acc_count, msg_count, thread_count);

    let mut threads = vec![];

    for i in 0..thread_count {
        threads.push(create_cycle_test_thread(config.clone(), accounts.clone(), timeout, msg_count, i));
    }

    for thread in threads {
        let _ = thread.join();
    }

    println!("The end");
}

const HELP: &str = r#"
Supported commands:
    balance <address>                       - get the account balance. If address is not provided current address is used
    create                                  - create new wallet account and set as current
    set                                     - set new wallet address
    send <address> <value>                  - send <value> grams to <address>
    create-limit <type> <value> <period>    - create limit
        type   - 0 for single transaction limits, 1 for arbitrary limits
        value  - limit value in grams
        period - limit period in days. Only applied to limit type 1
    create-limit <limit ID> <value> <period>    - change existing limit
        limit ID - limit ID returned by `create-limit` or `limits` function
        value    - new limit value in grams
        period   - new limit period in days. Only applied to limit type 1
    remove-limit <limit ID>                 - limit ID returned by `create-limit` or `limits` function
    get-limit <limit ID>                    - get one limit info
    limits                                  - list all existing wallet limits information
    version                                 - get version of the wallet contract
    cycle-test-full <accounts count> <timeout> <messages count> <threads count> - start a performance test - cyclically send founds between accounts
        accounts count - count of accounts
        timeout        - timeout in milliseconds between messages
        messages count - count of transfer messages
        threads count  - count of parallel working test threads
    cycle-test-init <accounts count>        - prepare cycle test data: deploy contracts and save accounts data
        accounts count - count of accounts
    cycle-test-run <timeout> <messages count> <threads count> - start a performance test - cyclically send founds between accounts prepared by `cycle-test-init` command
        timeout        - timeout in milliseconds between messages
        messages count - count of transfer messages
        threads count  - count of parallel working test threads
    exit                                    - exit program"#;

fn main() {
    let matches = App::new("TON Wallet client")
                        .version("0.1")
                        .author("Tonlabs <tonlabs.io>")
                        .arg(Arg::with_name("config")
                            .long("config")
                            .short("c")
                            .value_name("FILE")
                            .help("Sets a custom config file. Default is `config`")
                            .takes_value(true))
                        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config_file = matches.value_of("config").unwrap_or("config");
    println!("Using config file: `{}`", config_file);
    let config = std::fs::read_to_string(config_file).expect("Couldn't read config file");

    init_json(&config).expect("Couldn't establish connection");
    println!("Connection established");

    let mut current_address = if let Ok(address) = std::fs::read("last_address") {
        let address = MsgAddressInt::from_str(&String::from_utf8(address).unwrap()).unwrap();
        println!("Wallet address {}", serde_json::to_string(&address).unwrap());
        Some(address)
    } else {
        println!("Wallet address not assigned. Create new wallet");
        None
    };

    println!("Enter command");

    loop {
        println!("");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("error: unable to read user input");

        let params: Vec<&str> = input.split_whitespace().collect();

        if params.len() == 0 {
            continue;
        }

        match params[0].as_ref() {
            "help" => println!("{}", HELP),
            "balance" => call_get_balance(&current_address, &params[1..]),
            "create" => call_create(&mut current_address),
            "send" => call_send_transaction(&current_address, &params[1..]),
            "create-limit" => call_create_limit(&current_address, &params[1..]),
            "change-limit" => call_change_limit(&current_address, &params[1..]),
            "remove-limit" => call_remove_limit(&current_address, &params[1..]),
            "get-limit" => call_get_limit_by_id(&current_address, &params[1..]),
            "limits" => call_get_limits(&current_address),
            "version" => call_get_version(&current_address),
            "set" => set_address(&mut current_address, &params[1..]),
            "cycle-test-full" => cycle_test(config.clone(), &params[1..]),
            "cycle-test-init" => cycle_test_init(&params[1..]),
            "cycle-test-run" => cycle_test_run(config.clone(), &params[1..]),
            "exit" => break,
            _ => println!("Unknown command")
        }
    }
}
