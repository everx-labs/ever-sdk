extern crate ton_sdk;
extern crate hex;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate num_bigint;

use rand::{thread_rng, Rng};
use ton_block::{Message, MsgAddressExt, MsgAddressInt, InternalMessageHeader, Grams, 
    ExternalInboundMessageHeader, CurrencyCollection, Serializable, GetSetValueForVarInt,
    MessageProcessingStatus};
use tvm::bitstring::Bitstring;
use tvm::types::{AccountId};
use ed25519_dalek::Keypair;
use futures::Stream;
use sha2::Sha512;
use std::str::FromStr;
use num_traits::cast::ToPrimitive;

use ton_sdk::*;

const STD_CONFIG: &str = r#"
{
    "db_config": {
        "servers": ["142.93.137.28:28015"],
        "db_name": "blockchain"
    },
    "kafka_config": {
        "servers": ["142.93.137.28:9092"],
        "topic": "requests",
        "ack_timeout": 1000
    }
}"#;


const WALLET_ABI: &str = r#"{
    "ABI version" : 0,
    "functions" :	[{
            "inputs": [
                {"name": "recipient", "type": "bits256"},
                {"name": "value", "type": "duint"}
            ],
            "name": "sendTransaction",
            "signed": true,
            "outputs": [
                {"name": "transaction", "type": "uint64"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "type", "type": "uint8"},
                {"name": "value", "type": "duint"},
                {"name": "meta", "type": "bitstring"}
            ],
            "name": "createLimit",
            "signed": true,
            "outputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "error", "type": "int8"}
            ]
        }, {
            "inputs": [
                {"name": "limitId", "type": "uint8"},
                {"name": "value", "type": "duint"},
                {"name": "meta", "type": "bitstring"}
            ],
            "name": "changeLimitById",
            "signed": true,
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "removeLimit",
            "signed": true,
            "outputs": [{"name": "error", "type": "int8"}]
        }, {
            "inputs": [{"name": "limitId", "type": "uint8"}],
            "name": "getLimitById",
            "outputs": [
                {
                    "name": "limitInfo",
                    "type": "tuple",
                    "components": [
                        {"name": "value", "type": "duint"},
                        {"name": "type", "type": "uint8"},
                        {"name": "meta", "type": "bitstring"}
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
            "inputs": [{"name": "address", "type": "bits256" }],
            "name": "setSubscriptionAccount",
                    "signed": true,
            "outputs": []							
        }, {
            "inputs": [],
            "name": "getSubscriptionAccount",
            "outputs": [{"name": "address", "type": "bits256" }]							
        }
    ]
}
"#;

fn str_grams_to_nanorams(grams: &str) -> String {
    let grams = f64::from_str(grams).expect("Couldn't parse number");
    let nanograms = grams * 1000000000 as f64;
    format!("{}", nanograms as u64)
}

// Create message "from wallet" to transfer some funds 
// from one account to another
pub fn create_external_transfer_funds_message(src: AccountId, dst: AccountId, value: u128) -> Message {
    
    let mut rng = thread_rng();    
    let mut msg = Message::with_ext_in_header(
        ExternalInboundMessageHeader {
            src: MsgAddressExt::with_extern(&Bitstring::from(rng.gen::<u64>())).unwrap(),
            dst: MsgAddressInt::with_standart(None, 0, src.clone()).unwrap(),
            import_fee: Grams::default(),
        }
    );

    let mut balance = CurrencyCollection::default();
    balance.grams = Grams(value.into());

    let int_msg_hdr = InternalMessageHeader::with_addresses(
            MsgAddressInt::with_standart(None, 0, src).unwrap(),
            MsgAddressInt::with_standart(None, 0, dst).unwrap(),
            balance);

    msg.body = Some(int_msg_hdr.write_to_new_cell().unwrap().into());

    msg
}

fn deploy_contract_and_wait(code_file_name: &str, abi: &str, constructor_params: &str, key_pair: &Keypair) -> AccountId {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open(code_file_name).expect("Unable to open contract code file");

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id();

    // before deploying contract need to transfer some funds to its address
    //println!("Account ID to take some grams {}\n", account_id);
    let msg = create_external_transfer_funds_message(AccountId::from([0_u8; 32]), account_id.clone(), 100000000000);
    let changes_stream = Contract::send_message(msg).expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            //println!("message: {}  next state: {}", s.message_id.to_hex_string(), s.state);
            if s.message_state == MessageProcessingStatus::Finalized {
                tr_id = Some(s.message_id.clone());
                break;
            }
        }
    }
    tr_id.expect("Error: no transaction id");


    // call deploy method
    let changes_stream = Contract::deploy_json("constructor".to_owned(), constructor_params.to_owned(), abi.to_owned(), contract_image, Some(key_pair))
        .expect("Error deploying contract");

    // wait transaction id in message-status or 
    // wait message will done and find transaction with the message

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            //println!("next state: {:?}", s);
            if s.message_state == MessageProcessingStatus::Finalized {
                tr_id = Some(s.message_id.clone());
                break;
            }
        }
    }
    // contract constructor doesn't return any values so there are no output messages in transaction
    // so just check deployment transaction created
    let _tr_id = tr_id.expect("Error: no transaction id");

    account_id
}


fn call_contract_and_wait(address: AccountId, func: &str, input: &str, abi: &str, key_pair: Option<&Keypair>) -> String {

    let contract = Contract::load(address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    // call needed method
    let changes_stream = 
        Contract::call_json(contract.id(), func.to_owned(), input.to_owned(), abi.to_owned(), key_pair)
            .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            //println!("next state: {:?}", s);
            if s.message_state == MessageProcessingStatus::Finalized {
                tr_id = Some(s.message_id.clone());
                break;
            }
        }
    }
    let tr_id = tr_id.expect("Error: no transaction id");

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
    let responce = out_msg.body().expect("error unwrap out message body").into();

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

fn call_create(current_address: &mut Option<AccountId>) {
    println!("Creating new wallet account");

    // generate key pair
    let mut csprng = rand::rngs::OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);
   
    // deploy wallet
    let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);
    let str_address = hex::encode(wallet_address.as_slice());

    println!("Acoount created. Address {}", str_address);

    std::fs::write("last_address", wallet_address.as_slice()).expect("Couldn't save wallet address");
    std::fs::write(str_address, &keypair.to_bytes().to_vec()).expect("Couldn't save wallet key pair");

    *current_address = Some(wallet_address);
}

fn call_get_balance(current_address: &Option<AccountId>, params: &[&str]) {
    let address = if params.len() > 0 {
        AccountId::from(hex::decode(params[0]).unwrap())
    } else {
        if let Some(addr) = current_address.clone() {
            addr
        } else {
            println!("Current address not set");
            return;
        }
    };

    let contract = Contract::load(address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    let nanogram_balance = contract.balance_grams();
    let nanogram_balance = nanogram_balance.get_value().to_u128().expect("error cust grams to u128");
    let gram_balance = nanogram_balance as f64 / 1000000000f64;

    println!("Account balance {}", gram_balance);
}

#[derive(Deserialize)]
struct SendTransactionAnswer {
    transaction: String,
    error: String
}

fn call_send_transaction(current_address: &Option<AccountId>, params: &[&str]) {
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

    let pair = std::fs::read(hex::encode(address.as_slice())).expect("Couldn't read key pair");
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "sendTransaction", &str_params, WALLET_ABI, Some(&pair));


    let answer: SendTransactionAnswer = serde_json::from_str(&answer).unwrap();

    let transaction = u64::from_str_radix(&answer.transaction[2..], 16).expect("Couldn't parse transaction number");

    println!("Transaction ID {}", transaction);
}

#[derive(Deserialize)]
struct CreateLimitAnswer {
    limitId: String,
    error: String
}

fn call_create_limit(current_address: &Option<AccountId>, params: &[&str]) {
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

    let pair = std::fs::read(hex::encode(address.as_slice())).expect("Couldn't read key pair");
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "createLimit", &str_params, WALLET_ABI, Some(&pair));


    let answer: CreateLimitAnswer = serde_json::from_str(&answer).unwrap();

    let limit_id = u8::from_str_radix(&answer.limitId[2..], 16).expect("Couldn't parse limit ID");

    println!("Limit ID {}", limit_id);

}

#[derive(Deserialize)]
struct ChangeLimitAnswer {
    error: String
}

fn call_change_limit(current_address: &Option<AccountId>, params: &[&str]) {
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

    let pair = std::fs::read(hex::encode(address.as_slice())).expect("Couldn't read key pair");
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "changeLimitById", &str_params, WALLET_ABI, Some(&pair));


    let _answer: ChangeLimitAnswer = serde_json::from_str(&answer).unwrap();

    println!("Limit changed successfully");
}

fn call_remove_limit(current_address: &Option<AccountId>, params: &[&str]) {
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

    let pair = std::fs::read(hex::encode(address.as_slice())).expect("Couldn't read key pair");
    let pair = Keypair::from_bytes(&pair).expect("Couldn't restore key pair");

    let answer = call_contract_and_wait(address, "removeLimit", &str_params, WALLET_ABI, Some(&pair));


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

#[derive(Deserialize)]
struct GetLimitByIdAnswer {
    limitInfo: LimitInfo,
    error: String
}

fn call_get_limit_by_id(current_address: &Option<AccountId>, params: &[&str]) {
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

    let answer = call_contract_and_wait(address, "getLimitById", &str_params, WALLET_ABI, None);


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

#[derive(Deserialize)]
struct GetLimitsAnswer {
    list: Vec<String>,
    error: String
}

fn call_get_limits(current_address: &Option<AccountId>) {
    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let str_params = "{}".to_owned();

    let answer = call_contract_and_wait(address, "getLimits", &str_params, WALLET_ABI, None);


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

#[derive(Deserialize)]
struct GetVersionAnswer {
    version:Version,
    error: String
}

fn call_get_version(current_address: &Option<AccountId>) {
    let address = if let Some(addr) = current_address {
        addr.clone()
    } else {
        println!("Current address not set");
        return;
    };

    let str_params = "{}".to_owned();

    let answer = call_contract_and_wait(address, "getVersion", &str_params, WALLET_ABI, None);


    let answer: GetVersionAnswer = serde_json::from_str(&answer).unwrap();

    let major = u16::from_str_radix(&answer.version.major[2..], 16).unwrap();
    let minor = u16::from_str_radix(&answer.version.minor[2..], 16).unwrap();

    println!("Wallet version {}.{}", major, minor);
}

fn set_address(current_address: &mut Option<AccountId>, params: &[&str]) {
    if params.len() < 1 {
        println!("Not enough parameters");
        return;
    }

    if let Ok(vec) = hex::decode(params[0]) {
        if vec.len() != 32 {
            println!("Wrong address length. Address should be 32 bytes long");
            return;
        }

        if std::fs::read(params[0]).is_err() {
            println!("No key pair for this address. Can't work");
            return;
        }

        std::fs::write("last_address", vec.clone()).expect("Couldn't save wallet address");

        println!("New wallet address {}", params[0]);

        *current_address = Some(AccountId::from(vec));
    } else {
        println!("Couldn't parse address");
    }
}

fn cycle_test(params: &[&str]) {
    if params.len() < 2 {
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

    println!("Accounts creating...");
    let mut accounts = Vec::new();
    for _ in 0..acc_count {
        // generate key pair
        let mut csprng = rand::rngs::OsRng::new().unwrap();
        let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

        // deploy wallet
        let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);

        accounts.push((wallet_address, keypair));
    }

    println!("Transfer cycle...");

    for i in 0..msg_count {

        let (address_from, keypair) = &accounts[(i % acc_count) as usize];
        let (address_to, _) = &accounts[((i + 1) % acc_count) as usize];
        let value = 10;

        println!("Sending {} nanograms from {} to {}", value, address_from.to_hex_string(), address_to.to_hex_string());

        let str_params = format!("{{ \"recipient\" : \"x{}\", \"value\": \"{}\" }}", address_to.to_hex_string(), value);

        Contract::call_json(address_from.clone(), "sendTransaction".to_owned(), str_params.to_owned(), WALLET_ABI.to_owned(), Some(&keypair))
                .expect("Error calling contract method");

        std::thread::sleep(std::time::Duration::from_millis(timeout));
    }
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
    cycle-test <accounts count> <timeout> <messages count> - start a performance test - cyclically send founds between accounts
        accounts count - count of accounts
        timeout        - timeout in milliseconds between messages
        messages count - count of transfer messages
    exit                                    - exit program"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = if args.len() > 3 && args[2] == "-config" {
        std::fs::read_to_string(&args[3]).expect("Couldn't read config file")
    } else {
        if let Ok(string) = std::fs::read_to_string("config") {
            string
        } else {
            STD_CONFIG.to_owned()
        }
    };

    init_json(config).expect("Couldn't establish connection");
    println!("Connection established");

    let mut current_address: Option<AccountId> = None;

    if let Ok(address) = std::fs::read("last_address") {
        current_address = Some(AccountId::from(address));

        println!("Wallet address {}", hex::encode(current_address.clone().unwrap().as_slice()));
    } else {
        println!("Wallet address not assigned. Create new wallet");
    }

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
            "cycle-test" => cycle_test(&params[1..]),
            "exit" => break,
            _ => println!("Unknown command")
        }
    }
}
