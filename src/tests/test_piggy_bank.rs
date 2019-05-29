use abi_lib_dynamic::json_abi::decode_function_response;
use super::*;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use sha2::Sha512;
use tvm::types::AccountId;
use futures::Stream;
use ton_block::MessageProcessingStatus;

const SUBSCRIBE_CONTRACT_ABI: &str = r#"
{
    "ABI version": 0,
    "functions": [{
        "name": "constructor",
        "inputs": [{"name": "wallet", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "subscribe",
        "signed": true,
        "inputs": [
            {"name": "pubkey", "type": "bits256"},
            {"name": "to",     "type": "bits256"},
            {"name": "value",  "type": "duint"},
            {"name": "period", "type": "duint"}
        ],
        "outputs": [{"name": "subscriptionHash", "type": "bits256"}]
    }, {
        "name": "cancel",
        "signed": true,
        "inputs": [{"name": "subscriptionHash", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "executeSubscription",
        "inputs": [
            {"name": "subscriptionHash","type": "bits256"},
            {"name": "signature",       "type": "bits256"}
        ],
        "outputs": []
    }, {
        "name": "getSubscription",
        "inputs": [{"name": "subscriptionHash","type": "bits256"}],
        "outputs": [
            {"name": "to", "type": "bits256"},
            {"name": "amount", "type": "duint"},
            {"name": "period", "type": "duint"},
            {"name": "status", "type": "uint8"}
        ]
    }]
}"#;

const PIGGY_BANK_CONTRACT_ABI: &str = r#"
{
    "ABI version": 0,
    "functions": [{
        "name": "transfer",
        "signed": true,
        "inputs": [{"name": "to", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "getTargetAmount",
        "inputs": [],
        "outputs": [{"name": "amount", "type": "uint64"}]
    }, {
        "name": "getGoal",
        "inputs": [],
        "outputs": [{"name": "goal", "type": "uint8[]"}]
    }, {
        "name": "constructor",
        "inputs": [
				    {"name": "amount","type": "uint64"},
            {"name": "goal","type": "uint8[]"}
        ],
        "outputs": []
    }]
}"#;

// 123 Grams and "Some goal" as goal
const PIGGY_BANK_CONSTRUCTOR_PARAMS: &str = r#"
{
	"amount": 123,
	"goal": [83, 111, 109, 101, 32, 103, 111, 97, 108]
}"#;


const WALLET_ABI: &str = r#"{
	"ABI version" : 0,

	"functions" :	[
	    {
	        "inputs": [
	            {
	                "name": "recipient",
	                "type": "bits256"
	            },
	            {
	                "name": "value",
	                "type": "duint"
	            }
	        ],
	        "name": "sendTransaction",
					"signed": true,
	        "outputs": [
	            {
	                "name": "transaction",
	                "type": "uint64"
	            },
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
	    {
	        "inputs": [
						  {
	                "name": "type",
	                "type": "uint8"
	            },
							{
	                "name": "value",
	                "type": "duint"
	            },
							{
	                "name": "meta",
	                "type": "bitstring"
	            }
					],
	        "name": "createLimit",
					"signed": true,
	        "outputs": [
							{
	                "name": "limitId",
	                "type": "uint8"
	            },
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
	    {
	        "inputs": [
							{
	                "name": "limitId",
	                "type": "uint8"
	            },
							{
	                "name": "value",
	                "type": "duint"
	            },
							{
	                "name": "meta",
	                "type": "bitstring"
	            }
	        ],
	        "name": "changeLimitById",
					"signed": true,
	        "outputs": [
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
			{
	        "inputs": [
							{
	                "name": "limitId",
	                "type": "uint8"
	            }
	        ],
	        "name": "removeLimit",
					"signed": true,
	        "outputs": [
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
			{
	        "inputs": [
							{
	                "name": "limitId",
	                "type": "uint8"
	            }
	        ],
	        "name": "getLimitById",
	        "outputs": [
							{
									"name": "limitInfo",
					        "type": "tuple",
					        "components": [
											{
					                "name": "value",
					                "type": "duint"
					            },
											{
					                "name": "type",
					                "type": "uint8"
					            },
											{
					                "name": "meta",
					                "type": "bitstring"
					            }
									]
							},
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
			{
	        "inputs": [],
	        "name": "getLimits",
	        "outputs": [
							{
									"name": "list",
					        "type": "uint8[]"
							},
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
			{
	        "inputs": [],
	        "name": "getVersion",
	        "outputs": [
							{
									"name": "version",
					        "type": "tuple",
					        "components": [
											{
					                "name": "major",
					                "type": "uint16"
					            },
											{
					                "name": "minor",
					                "type": "uint16"
					            }
									]
							},
							{
	                "name": "error",
	                "type": "int8"
	            }
	        ]
	    },
			{
	        "inputs": [],
	        "name": "getBalance",
	        "outputs": [
							{
	                "name": "balance",
	                "type": "uint64"
	            }
	        ]
	    },
			{
	        "inputs": [],
	        "name": "constructor",
	        "outputs": []							
	    },
			{
	        "inputs": [{"name": "address", "type": "bits256" }],
	        "name": "setSubscriptionAccount",
					"signed": true,
	        "outputs": []							
	    },
			{
	        "inputs": [],
	        "name": "getSubscriptionAccount",
	        "outputs": [{"name": "address", "type": "bits256" }]							
	    }
	]
}
"#;

fn call_contract(address: AccountId, func: &str, input: &str, abi: &str, key_pair: &Keypair) {

    let contract = Contract::load(address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    // call needed method
    let changes_stream = Contract::call_json(contract.id(), func.to_owned(), input.to_owned(), abi.to_owned(), Some(&key_pair))
        .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
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
    let _tr = Transaction::load(tr_id)
        .expect("Error calling load Transaction")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Transaction")
        .expect("Error unwrap result while loading Transaction");
}

fn call_contract_and_wait(address: AccountId, func: &str, input: &str, abi: &str, key_pair: &Keypair) -> String {

    let contract = Contract::load(address)
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    // call needed method
    let changes_stream = Contract::call_json(contract.id(), func.to_owned(), input.to_owned(), abi.to_owned(), Some(&key_pair))
        .expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
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
        .expect("Error unwrap returned Transaction");

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
    let response = out_msg.body().expect("erro unwrap out message body").into();


    // decode the body by ABI
    let result = decode_function_response(abi.to_owned(), func.to_owned(), response)
        .expect("Error decoding result");

    println!("Contract call result: {}\n", result);

	result

    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body

}

fn init_node_connection() {
       let config_json = r#"
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
    init_json(config_json.into()).unwrap(); 
}

fn deploy_contract_and_wait(code_file_name: &str, abi: &str, constructor_params: &str, key_pair: &Keypair) -> AccountId {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/".to_owned() + code_file_name).expect("Unable to open contract code file");

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id();

    // before deploying contract need to transfer some funds to its address
    println!("Account ID to take some grams {}\n", account_id);
    let msg = create_external_transfer_funds_message(AccountId::from([0_u8; 32]), account_id.clone(), 100);
    let changes_stream = Contract::send_message(msg).expect("Error calling contract method");

    // wait transaction id in message-status 
    let mut tr_id = None;
    for state in changes_stream.wait() {
        if let Err(e) = state {
            panic!("error next state getting: {}", e);
        }
        if let Ok(s) = state {
            println!("next state: {:?}", s);
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
            println!("next state: {:?}", s);
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

#[test]
fn full_test_piggy_bank() {
    
	// connect to node
    init_node_connection();

	println!("Connection to node established\n");

	// generate key pair
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);
   
	// deploy wallet
    let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);

	println!("Wallet contract deployed. Account address {}\n", wallet_address);

	// deploy piggy bank
	let piggy_bank_address = deploy_contract_and_wait("Piggybank.tvc", PIGGY_BANK_CONTRACT_ABI, PIGGY_BANK_CONSTRUCTOR_PARAMS, &keypair);

	println!("Piggy bank contract deployed. Account address {}\n", piggy_bank_address);

	// deploy subscription
	let wallet_address_str = hex::encode(wallet_address.as_slice());
	let subscription_constructor_params = format!("{{ \"wallet\" : \"x{}\" }}", wallet_address_str);
	let subscripition_address = deploy_contract_and_wait("Subscription.tvc", SUBSCRIBE_CONTRACT_ABI, &subscription_constructor_params, &keypair);

	println!("Subscription contract deployed. Account address {}\n", subscripition_address);

	// call setSubscriptionAccount in wallet
	let subscripition_address_str = hex::encode(subscripition_address.as_slice());
	let set_subscription_params = format!("{{ \"address\" : \"x{}\" }}", subscripition_address_str);

	let _set_subscription_answer = call_contract(wallet_address, "setSubscriptionAccount", &set_subscription_params, WALLET_ABI, &keypair);

	println!("Subscription address added to wallet.\n");

	// call subscribe in subscription
	let piggy_bank_address_str = hex::encode(piggy_bank_address.as_slice());
	let pubkey_str = hex::encode(keypair.public.as_bytes());
	let subscribe_params = format!("{{ \"pubkey\" : \"x{}\", \"to\": \"x{}\", \"value\" : 123, \"period\" : 456 }}", pubkey_str, piggy_bank_address_str);

	let _subscribe_answer = call_contract_and_wait(subscripition_address, "subscribe", &subscribe_params, SUBSCRIBE_CONTRACT_ABI, &keypair);

	println!("New subscription added.\n");
}


use rand::{thread_rng, Rng};
use ton_block::{Message, MsgAddressExt, MsgAddressInt, InternalMessageHeader, Grams, 
    ExternalInboundMessageHeader, CurrencyCollection, Serializable};
use tvm::bitstring::Bitstring;

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