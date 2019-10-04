use super::*;
use ed25519_dalek::Keypair;
use futures::Stream;
use rand::rngs::OsRng;
use sha2::Sha512;
use tvm::block::{
    MessageId,
    TransactionId,
    TransactionProcessingStatus,
    Message as TvmMessage,
    Deserializable
};
use tvm::types::AccountId;

const WORKCHAIN: i32 = 0;

lazy_static! {
    static ref GIVER_ADDRESS: AccountAddress = 
        AccountAddress::from_str("ce709b5bfca589eb621b5a5786d0b562761144ac48f59e0b0d35ad0973bcdb86")
            .unwrap();
}

const GIVER_DEPLOY_MSG: &'static str = "b5ee9c7241045301000000091e00035188019ce136b7f94b13d6c436b4af0da16ac4ec22895891eb3c161a6b5a12e779b70c11801590214db007020100000101c0030203cf2006040101de050003d0200041d80000000000000000000000000000000000000000000000000000000000000004017eff00f80089f40521c3018e158020fefe0173656c6563746f725f6a6d705f30f4a08e1b8020f40df2b48020fefc0173656c6563746f725f6a6d70f4a1f233e2080101c0090201200b0a0018ff8210d0a6aff0f001dcf0010202de520c0101300d020120290e0201201c0f0201201710020120161102016a131200a4b2e0784dfefe016765745f6d73675f7075626b65797021c7029670313171db308e2921d520c7019770045f0471db30e020810200821059b09094f0013321f901202225f91020f2a85f0470e2dc203131db300201201514003db10c31ef0421d9b9aa13e0024379e57840459193a10420ab7e793de002be050015b0fcdca6db91e80193daa90039b99b9aa13fdf602cecae8bec4c2d8c2dcc6cbda8ede20de2ede21b661002015819180073b6408f57bfbec0595b98dbd91959dc985b5cc81c2f6385882d80dde81e2a410808f2c0ccc848c89e2a33c04ccc255c08b2c0ccb8884c4c76cc200201201b1a00a1b4348f4cff7f00b1b430b733b2afb0b9392fb632b710c0107a474918d248b87110115f470a470890115e59906e18529011c0107a2d9819b8734f91b759cd119138d09240107a0b19ef7111022f826d984000efb45357f87f7e80b6b0b4b72fb2bc3a32b93730b610c1084c2d05c2780091c1087d703c26f80092638159ca126a189aef13939191c10806828f39f800926380c711ff7e00b6b9b3afb4b9afb2b6b83a3cb6e41090fa0064989076aa383838aac92f856d987012410808216251f8009090b82ac92f856d9840020120221d020120211e020120201f00bdb72bee62ffbf405b585a5b97da5b9d195c9b985b086084261682e13c004989889c20840341479cfc0048f1c02387485c2ea7e084171fb881dc085c155c97c236cc381c1c1c555897c1f6cc3808e0840410b128fc0048485c156097c276cc200091b634ead34875d24808afa74888b5c04d0809154c57c136cc38088875c60d08f50d49345b4d48340809496875c60cb20908738c4848738c4832740c4809f5c04c8809156057c276cc20001fb990214da610421f0fcdca7e003b6610020120282302015827240201202625005cb3615691fefc0173656e645f6578745f6d7367f8258210881607bdf001222222821065ffe8e7f0012070fb005f040048b3167d2e8101008210b0d3ab4df00180408210b0d3ab4df001308210f98618f7f001db300091b42d05c27f7e00b3b2ba2fb9b931afb0b2323910681069801910385eccb838382a912f81ed987010b8eb909910e980199169801a11c005c1082cd8484a78009111112aa92f836d98400019b902c0f7bda8ede20de31b6610020120402a020120352b020120302c0201202e2d009db754e559ffbf005cd95b9917da5b9d17db5cd9f21c8872c04c5c0872c00c5c0872c1cc488872ffcc4832740c6084220581ef7c007e08c9485c1c1c0a0a0a1c1c20841d23ac99bc00481c3ec017c22001edb723ac99bfbe00589d5a5b191b5cd9f21c0872c00c484872c00c488872c00c5c0872c00c48c8738c4908738c4809a08436408f57bc004c5c0872c00c4809e08436408f57bc004c480a208436408f57bc004c4a0872cfcc4a4872c7cc5c0872c00c4833cd4af5d25c6808486f265c08f2c00ccb08f38ce02f00348e107123cb0033c82d21ce3120c924cc3430e222c90d5f0ddb30020120343102012033320051b55fa4d8ff7e80b3b2ba2fb9b2b6332fb0b232394108440b03def800c005c1082cd8484a7800ed98400031b50968553876a3b788e87a02bc7a0749e9ffe8c8b8716d984000c3b77ffa39ffbf40589d5a5b1917d95e1d17db5cd9f21cc872c04c4848738c5c0872c04c488872cfcc5c0872c7cc5c0872c00c4833cd4935d25c6808486f265c08f2c00cc948f38ce3841c48f2c00cf20988738c483249330d0c3888b24197c1b6cc200201203f360201203c370201203b380201483a390049b184338bfdfc02e6cadcc8bed2dce8bedae6cebe64e04247024e210420faa72acfe002be05000db0fdc40e61b6610027b4d8484a1090eb909069ff99189001af81ed98400201203e3d003fb4df9e4f7f7d00b9b2b7322fb3b930b6b9b8109192c1083ea9cab3f800af81c00031b4ef31a7b8f6a3b788e87a02bc7a0749e9ffe8c8b8716d984000b7b8730fd55fdf802c8cac6dec8cabec2e4e4c2f2438e032e43a86641a06661bc43a6026640e375e5c0c845a63e6847e808430041e91d24634922e1c4464375e5c0c9fdfe02c8cac6dec8cabec2e4e4c2f2beded64444aac2be0fb66100201204841020120454202015844430075b409c48810c0107a474918d248b87138471010115cd9906e18111092c0107a074898cbe438016780e4e8711013671b185238731811822f826d98400039b488f3fa64129091e780989064e81890129292fa0b1a11832f836d984002012047460039b70b82c03fbf4059d95d17dc985b9917dcd959593b51dbc41bc5b6cc200031b7b7811e1cbb51dbc4743d015e3d03a4f4fff4645c38b6cc200201204e490201204b4a0067b6c209cf3fbf00595b98dbd91957d85c9c985e5c48f2c04cc860083d23a48c69245c38880932c7cd08893d000d08c117c136cc200201624d4c008fb09bed1443ae9240457d3a4445ae00684048aa62be09b661c04443ae306847a86a49a2da6a41a0404a4b43ae30659048439c6242439c624193a062404fae00644048ab02be13b6610031b085894641a60e6440e175e56e43a63e664442aa42be07b661020276514f01ffb00a3ce7fdfe02e6e8dee4cabee6d2cedcc2e8eae4cada42e044f102020104207223cfe9e0026244e244f102020104207223cfe9e0026246e444f102020104207223cfe9e0026248e644f102020104207223cfe9e002630420585c1601e0030421d9b9aa13e00390424396fe624193a0630420dd7e9363e00240e84cf102020150006a82103911e7f4f0013521752678f416352376267881010082103911e7f4f00135c82521f4003120c931ed4720226f8c3120ed575f0b0055b0e032244445ae306847a86a49a2da6a41a06a4847ae306d9046439c6242439c624193a04eaac2be0fb661001b20842f2bee62fc00773c0076cc20e7af7b3d";

const GIVER_ABI: &str = r#"
{
    "ABI version": 0,
    "functions": [{
        "name": "constructor",
        "inputs": [],
        "outputs": []
    }, {
        "name": "sendGrams",
        "inputs": [
            {"name":"dest","type":"uint256"},
            {"name":"amount","type":"uint64"}
        ],
        "outputs": []
    }]
}"#;

pub const SUBSCRIBE_CONTRACT_ABI: &str = r#"
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
    let config_json =  if 1 == 0 {
        r#"
        {
            "queries_config": {
                "queries_server": "https://services.tonlabs.io/graphql",
                "subscriptions_server": "wss://services.tonlabs.io/graphql"
            },
            "requests_config": {
                "requests_server": "https://services.tonlabs.io/topics/requests"
            }
        }"#
    } else {
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
    };

        
    init_json(Some(WORKCHAIN), config_json.into()).unwrap();
}

fn is_message_done(status: TransactionProcessingStatus) -> bool {
    (status == TransactionProcessingStatus::Preliminary) ||
    (status == TransactionProcessingStatus::Proposed) ||
    (status == TransactionProcessingStatus::Finalized)
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
    wait_message_processed(Contract::subscribe_updates(id.clone()).unwrap())
}

fn check_giver() {
    let contract = Contract::load(GIVER_ADDRESS.clone())
        .expect("Error calling load Contract")
        .wait()
        .next()
        .expect("Error unwrap stream next while loading Contract")
        .expect("Error unwrap result while loading Contract");

    if contract.is_none() {
        println!("No giver now. Deploy");

        let mut cursor = std::io::Cursor::new(hex::decode(GIVER_DEPLOY_MSG).unwrap());
        let cell = tvm::cells_serialization::deserialize_cells_tree(&mut cursor).unwrap().remove(0);
        let msg: TvmMessage = TvmMessage::construct_from(&mut cell.into()).unwrap();

        // wait transaction id in message-status
        let tr_id = wait_message_processed(Contract::send_message(msg).unwrap());

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

        println!("Giver deployed\n");
    }
}

pub fn get_grams_from_giver(account_id: AccountId) {

    check_giver();

    let transaction = call_contract(
        GIVER_ADDRESS.get_account_id().unwrap(),
        "sendGrams",
        &json!({
           "dest": format!("0x{:x}", account_id),
           "amount": 1_000_000_000u64
        }).to_string(),
        GIVER_ABI,
        None);

    transaction.out_messages_id().iter().for_each(|msg_id| {
        wait_message_processed_by_id(msg_id.clone());
    });
}

fn deploy_contract_and_wait(code_file_name: &str, abi: &str, constructor_params: &str, key_pair: &Keypair) -> AccountId {
    // read image from file and construct ContractImage
    let mut state_init = std::fs::File::open("src/tests/".to_owned() + code_file_name).expect("Unable to open contract code file");

    let contract_image = ContractImage::from_state_init_and_key(&mut state_init, &key_pair.public).expect("Unable to parse contract code file");

    let account_id = contract_image.account_id();

    get_grams_from_giver(account_id.clone());

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

fn call_contract(address: AccountId, func: &str, input: &str, abi: &str, key_pair: Option<&Keypair>) -> Transaction {
    // call needed method
    let changes_stream = Contract::call_json(address.into(), func.to_owned(), input.to_owned(), abi.to_owned(), key_pair)
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

    tr
}

fn call_contract_and_wait(address: AccountId, func: &str, input: &str, abi: &str, key_pair: Option<&Keypair>)
    -> (String, Transaction)
{
    // call needed method
    let changes_stream =
        Contract::call_json(address.into(), func.to_owned(), input.to_owned(), abi.to_owned(), key_pair)
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

    if tr.tr().is_aborted() {
        panic!("transaction aborted!\n\n{}", serde_json::to_string_pretty(tr.tr()).unwrap())
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
            && abi_function.is_my_message(msg.body().expect("No body")).expect("error is_my_message")
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

    (result, tr)

    // this way it is need:
    // 1. message status with transaction id or transaction object with in-message id
    // 2. transaction object with out messages ids
    // 3. message object with body
}

fn local_contract_call(address: AccountId, func: &str, input: &str, abi: &str, key_pair: Option<&Keypair>) -> String {

    let contract = Contract::load(address.into())
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
        let msg = crate::Message::with_msg(msg);
        if msg.msg_type() == MessageType::ExternalOutbound {
            return Contract::decode_function_response_json(
                abi.to_owned(), func.to_owned(), msg.body().expect("Message has no body"))
                    .expect("Error decoding result");
        }
    }

    panic!("No output messages")
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
    //let get_goal_answer = call_contract_and_wait(piggy_bank_address.clone(), "getGoal", "{}", PIGGY_BANK_CONTRACT_ABI, None);
    let get_goal_answer = local_contract_call(piggy_bank_address.clone(), "getGoal", "{}", PIGGY_BANK_CONTRACT_ABI, None);
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

	let _set_subscription_answer = call_contract(wallet_address, "setSubscriptionAccount", &set_subscription_params, WALLET_ABI, Some(&keypair));

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

    uninit();
}
