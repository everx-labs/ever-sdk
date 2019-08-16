use super::*;
use ed25519_dalek::Keypair;
use futures::Stream;
use rand::{thread_rng, Rng};
use rand::rngs::OsRng;
use sha2::Sha512;
use tvm::block::{
    Message, MsgAddressExt, MsgAddressInt, InternalMessageHeader, Grams, 
    MessageProcessingStatus, MessageId, TransactionId,
    ExternalInboundMessageHeader, CurrencyCollection, Serializable
};
use tvm::stack::{BuilderData, IBitstring};
use tvm::types::AccountId;

const WORKCHAIN: i32 = 0;
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
            {"name": "subscriptionId", "type": "bits256"},
            {"name": "pubkey", "type": "bits256"},
            {"name": "to",     "type": "bits256"},
            {"name": "value",  "type": "duint"},
            {"name": "period", "type": "duint"}
        ],
        "outputs": [{"name": "subscriptionHash", "type": "bits256"}]
    }, {
        "name": "cancel",
        "signed": true,
        "inputs": [{"name": "subscriptionId", "type": "bits256"}],
        "outputs": []
    }, {
        "name": "executeSubscription",
        "inputs": [
            {"name": "subscriptionId",  "type": "bits256"},
            {"name": "signature",       "type": "bits256"}
        ],
        "outputs": []
    }, {
        "name": "getSubscription",
        "inputs": [{"name": "subscriptionId","type": "bits256"}],
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

fn init_node_connection() {
    let config_json = r#"
        {
            "queries_server": "services.tonlabs.io:4000/graphql",
            "requests_server": "services.tonlabs.io/topics/requests"
        }"#;
    init_json(Some(WORKCHAIN), config_json.into()).unwrap();
}

fn is_message_done(status: MessageProcessingStatus) -> bool {
    (status == MessageProcessingStatus::Preliminary) ||
    (status == MessageProcessingStatus::Proposed) ||
    (status == MessageProcessingStatus::Finalized)
}

fn wait_message_processed(changes_stream: Box<dyn Stream<Item = ContractCallState, Error = SdkError>>) -> TransactionId {
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

fn wait_message_processed_by_id(id: MessageId)-> TransactionId {
    let msg = crate::Message::load(id.clone())
        .expect("Error load message")
        .wait()
        .next();

    if msg.is_some() {
        let s = msg.expect("Error unwrap stream next while loading Message")
            .expect("Error unwrap result while loading Message")
            .expect("Error unwrap returned Message");
        println!("{} : {:?}", s.id().to_hex_string(), s.status());
        if is_message_done(s.status()) {
            return s.id().clone();
        }
    }

    wait_message_processed(Contract::subscribe_updates(id.clone()).unwrap())
}

fn deploy_contract_and_wait(code_file_name: &str, abi: &str, constructor_params: &str, key_pair: &Keypair) -> AccountId {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/".to_owned() + code_file_name).expect("Unable to open contract code file");

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id();

    // before deploying contract need to transfer some funds to its address
    //println!("Account ID to take some grams {}\n", account_id.to_hex_string());
    let msg = create_external_transfer_funds_message(AccountId::from([0; 32]), account_id.clone(), 100000000000);
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

fn call_contract(address: AccountId, func: &str, input: &str, abi: &str, key_pair: &Keypair) {

    let contract = Contract::load(address.into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    // call needed method
    let changes_stream = Contract::call_json(contract.id().into(), func.to_owned(), input.to_owned(), abi.to_owned(), Some(&key_pair))
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
        .expect("Error unwrap returned Transaction");

    if tr.tr().is_aborted() {
        panic!("transaction aborted!\n\n{}", serde_json::to_string_pretty(tr.tr()).unwrap())
    }
}

fn call_contract_and_wait(address: AccountId, func: &str, input: &str, abi: &str, key_pair: Option<&Keypair>) -> String {

    let contract = Contract::load(address.into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");

    // call needed method
    let changes_stream =
        Contract::call_json(contract.id().into(), func.to_owned(), input.to_owned(), abi.to_owned(), key_pair)
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
    let responce = out_msg.body().expect("error unwrap out message body").into();

    //println!("response {}", responce);

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

#[test]
fn full_test_piggy_bank() {

    // connect to node
    init_node_connection();

    println!("Connection to node established\n");

    /*let contract = Contract::load(AccountId::from([0; 32]).into())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract")
        .expect("Error unwrap contract while loading Contract");*/

	// generate key pair
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let now = std::time::Instant::now();

	// deploy wallet
    println!("Wallet contract deploying...\n");
    let wallet_address = deploy_contract_and_wait("Wallet.tvc", WALLET_ABI, "{}", &keypair);
	println!("Wallet contract deployed. Account address {}\n", wallet_address.to_hex_string());

	// deploy piggy bank
    println!("Piggy bank contract deploying...\n");
	let piggy_bank_address = deploy_contract_and_wait("Piggybank.tvc", PIGGY_BANK_CONTRACT_ABI, PIGGY_BANK_CONSTRUCTOR_PARAMS, &keypair);
	println!("Piggy bank contract deployed. Account address {}\n", piggy_bank_address.to_hex_string());

    // get goal from piggy
    println!("Get goal from piggy...\n");
    let get_goal_answer = call_contract_and_wait(piggy_bank_address.clone(), "getGoal", "{}", PIGGY_BANK_CONTRACT_ABI, None);
    println!("piggy answer {}", get_goal_answer);

	// deploy subscription

    println!("Subscription contract deploying...\n");
	let wallet_address_str = wallet_address.to_hex_string();
	let subscription_constructor_params = format!("{{ \"wallet\" : \"x{}\" }}", wallet_address_str);
	let subscripition_address = deploy_contract_and_wait("Subscription.tvc", SUBSCRIBE_CONTRACT_ABI, &subscription_constructor_params, &keypair);
	println!("Subscription contract deployed. Account address {}\n", subscripition_address.to_hex_string());


    // call setSubscriptionAccount in wallet
    println!("Adding subscription address to the wallet...\n");
	let subscripition_address_str = subscripition_address.to_hex_string();
	let set_subscription_params = format!("{{ \"address\" : \"x{}\" }}", subscripition_address_str);

	let _set_subscription_answer = call_contract(wallet_address, "setSubscriptionAccount", &set_subscription_params, WALLET_ABI, &keypair);

	println!("Subscription address added to the wallet.\n");

	// call subscribe in subscription
    println!("Adding subscription 1...\n");
    let subscr_id_str = hex::encode(&[0x11; 32]);
	let piggy_bank_address_str = piggy_bank_address.to_hex_string();
	let pubkey_str = hex::encode(keypair.public.as_bytes());
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"x{}\", \"pubkey\" : \"x{}\", \"to\": \"x{}\", \"value\" : 123, \"period\" : 456 }}",
        subscr_id_str,
        &pubkey_str,
        &piggy_bank_address_str,
    );

	let _subscribe_answer = call_contract_and_wait(subscripition_address.clone(), "subscribe", &subscribe_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
	println!("Subscription 1 added.\n");

    	// call subscribe in subscription
    println!("Adding subscription 2...\n");
    let subscr_id_str = hex::encode(&[0x22; 32]);
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"x{}\", \"pubkey\" : \"x{}\", \"to\": \"x{}\", \"value\" : 5000000000, \"period\" : 86400 }}",
        subscr_id_str,
        &pubkey_str,
        &piggy_bank_address_str,
    );
	let _subscribe_answer = call_contract_and_wait(subscripition_address.clone(), "subscribe", &subscribe_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
	println!("Subscription 2 added.\n");

    println!("Call getSubscription with id {}\n", &subscr_id_str);
    let get_params = format!("{{ \"subscriptionId\" : \"x{}\" }}", &subscr_id_str);
    call_contract_and_wait(subscripition_address, "getSubscription", &get_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
    println!("getSubscription called.\n");

    let t = now.elapsed();
	println!("Time: sec={}.{:06} ", t.as_secs(), t.subsec_micros());
}

// Create message "from wallet" to transfer some funds 
// from one account to another
pub fn create_external_transfer_funds_message(src: AccountId, dst: AccountId, value: u128) -> Message {
    
    let mut rng = thread_rng();    
    let mut builder = BuilderData::new();
    builder.append_u64(rng.gen::<u64>()).unwrap();
    let mut msg = Message::with_ext_in_header(
        ExternalInboundMessageHeader {
            src: MsgAddressExt::with_extern(builder.into()).unwrap(),
            dst: MsgAddressInt::with_standart(None, WORKCHAIN as i8, src.clone()).unwrap(),
            import_fee: Grams::default(),
        }
    );

    let mut balance = CurrencyCollection::default();
    balance.grams = Grams(value.into());

    let int_msg_hdr = InternalMessageHeader::with_addresses(
        MsgAddressInt::with_standart(None, WORKCHAIN as i8, src).unwrap(),
        MsgAddressInt::with_standart(None, WORKCHAIN as i8, dst).unwrap(),
        balance
    );

    *msg.body_mut() = Some(int_msg_hdr.write_to_new_cell().unwrap().into());
    msg

}
