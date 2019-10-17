use super::*;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use sha2::Sha512;
use tests_common::*;

#[test]
fn full_test_piggy_bank() {

    // connect to node
    init_node_connection();

    println!("Connection to node established\n");

	// generate key pair
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let now = std::time::Instant::now();

	// deploy wallet
    println!("Wallet contract deploying...\n");
    let wallet_address = deploy_contract_and_wait("LimitWallet.tvc", WALLET_ABI, "{}", &keypair);
	println!("Wallet contract deployed. Account address {}\n", wallet_address);

	// deploy piggy bank
    println!("Piggy bank contract deploying...\n");
	let piggy_bank_address = deploy_contract_and_wait("Piggy.tvc", PIGGY_BANK_CONTRACT_ABI, PIGGY_BANK_CONSTRUCTOR_PARAMS, &keypair);
	println!("Piggy bank contract deployed. Account address {}\n", piggy_bank_address);

    // get goal from piggy
    println!("Get goal from piggy...\n");
    let (get_goal_answer, _) = call_contract_and_wait(&piggy_bank_address, "getGoal", "{}".to_string(), PIGGY_BANK_CONTRACT_ABI, None);
    //let get_goal_answer = local_contract_call(piggy_bank_address.clone(), "getGoal", "{}", PIGGY_BANK_CONTRACT_ABI, None);
    println!("piggy answer {}", get_goal_answer);

	// deploy subscription

    println!("Subscription contract deploying...\n");
	let subscription_constructor_params = format!("{{ \"wallet\" : \"0x{:x}\" }}", wallet_address.get_account_id().unwrap());
	let subscripition_address = deploy_contract_and_wait("Subscription.tvc", SUBSCRIBE_CONTRACT_ABI, &subscription_constructor_params, &keypair);
	println!("Subscription contract deployed. Account address {}\n", subscripition_address);


    // call setSubscriptionAccount in wallet
    println!("Adding subscription address to the wallet...\n");
	let set_subscription_params = format!("{{ \"addr\" : \"0x{:x}\" }}", subscripition_address.get_account_id().unwrap());

	let _set_subscription_answer = call_contract(&wallet_address, "setSubscriptionAccount", set_subscription_params, WALLET_ABI, Some(&keypair));

	println!("Subscription address added to the wallet.\n");

	// call subscribe in subscription
    println!("Adding subscription 1...\n");
    let subscr_id_str = hex::encode(&[0x11; 32]);
	let pubkey_str = hex::encode(keypair.public.as_bytes());
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"0x{}\", \"pubkey\" : \"0x{}\", \"to\": \"0x{:x}\", \"value\" : 123, \"period\" : 456 }}",
        subscr_id_str,
        &pubkey_str,
        piggy_bank_address.get_account_id().unwrap(),
    );

	call_contract(&subscripition_address, "subscribe", subscribe_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
	println!("Subscription 1 added.\n");

    	// call subscribe in subscription
    println!("Adding subscription 2...\n");
    let subscr_id_str = hex::encode(&[0x22; 32]);
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"0x{}\", \"pubkey\" : \"0x{}\", \"to\": \"0x{:x}\", \"value\" : 5000000000, \"period\" : 86400 }}",
        subscr_id_str,
        &pubkey_str,
        piggy_bank_address.get_account_id().unwrap(),
    );
	call_contract(&subscripition_address, "subscribe", subscribe_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
	println!("Subscription 2 added.\n");

    println!("Call getSubscription with id {}\n", &subscr_id_str);
    let get_params = format!("{{ \"subscriptionId\" : \"0x{}\" }}", &subscr_id_str);
    call_contract_and_wait(&subscripition_address, "getSubscription", get_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
    println!("getSubscription called.\n");

    let t = now.elapsed();
	println!("Time: sec={}.{:06} ", t.as_secs(), t.subsec_micros());

    uninit();
}

pub const SUBSCRIBE_CONTRACT_ABI: &str = r#"
{
	"ABI version": 1,
	"functions": [
		{
			"name": "constructor",
			"inputs": [
				{"name":"wallet","type":"uint256"}
			],
			"outputs": [
			]
		},
		{
			"name": "getWallet",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint256"}
			]
		},
		{
			"name": "getSubscription",
			"inputs": [
				{"name":"subscriptionId","type":"uint256"}
			],
			"outputs": [
				{"components":[{"name":"pubkey","type":"uint256"},{"name":"to","type":"uint256"},{"name":"value","type":"uint64"},{"name":"period","type":"uint32"},{"name":"start","type":"uint32"},{"name":"status","type":"uint8"}],"name":"value0","type":"tuple"}
			]
		},
		{
			"name": "subscribe",
			"inputs": [
				{"name":"subscriptionId","type":"uint256"},
				{"name":"pubkey","type":"uint256"},
				{"name":"to","type":"uint256"},
				{"name":"value","type":"uint64"},
				{"name":"period","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "cancel",
			"inputs": [
				{"name":"subscriptionId","type":"uint256"}
			],
			"outputs": [
			]
		},
		{
			"name": "executeSubscription",
			"inputs": [
				{"name":"subscriptionId","type":"uint256"}
			],
			"outputs": [
			]
		}
	],
	"events": [
	],
	"data": [
		{"key":100,"name":"mywallet","type":"uint256"}
	]
} "#;

pub const PIGGY_BANK_CONTRACT_ABI: &str = r#"
{
	"ABI version": 1,
	"functions": [
		{
			"name": "constructor",
			"inputs": [
				{"name":"amount","type":"uint64"},
				{"name":"goal","type":"uint8[]"}
			],
			"outputs": [
			]
		},
		{
			"name": "getGoal",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint8[]"}
			]
		},
		{
			"name": "getTargetAmount",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint64"}
			]
		},
		{
			"name": "transfer",
			"inputs": [
				{"name":"to","type":"uint256"}
			],
			"outputs": [
			]
		}
	],
	"events": [
	],
	"data": [
		{"key":100,"name":"targetGoal","type":"uint8[]"},
		{"key":101,"name":"targetAmount","type":"uint64"}
	]
} "#;

// 123 Grams and "Some goal" as goal
const PIGGY_BANK_CONSTRUCTOR_PARAMS: &str = r#"
{
	"amount": 123,
	"goal": [83, 111, 109, 101, 32, 103, 111, 97, 108]
}"#;


const WALLET_ABI: &str = r#"
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
		},
		{
			"name": "createOperationLimit",
			"inputs": [
				{"name":"value","type":"uint256"}
			],
			"outputs": [
				{"name":"value0","type":"uint256"}
			]
		},
		{
			"name": "createArbitraryLimit",
			"inputs": [
				{"name":"value","type":"uint256"},
				{"name":"period","type":"uint32"}
			],
			"outputs": [
				{"name":"value0","type":"uint64"}
			]
		},
		{
			"name": "changeLimit",
			"inputs": [
				{"name":"limitId","type":"uint64"},
				{"name":"value","type":"uint256"},
				{"name":"period","type":"uint32"}
			],
			"outputs": [
			]
		},
		{
			"name": "deleteLimit",
			"inputs": [
				{"name":"limitId","type":"uint64"}
			],
			"outputs": [
			]
		},
		{
			"name": "getLimit",
			"inputs": [
				{"name":"limitId","type":"uint64"}
			],
			"outputs": [
				{"components":[{"name":"value","type":"uint256"},{"name":"period","type":"uint32"},{"name":"ltype","type":"uint8"},{"name":"spent","type":"uint256"},{"name":"start","type":"uint32"}],"name":"value0","type":"tuple"}
			]
		},
		{
			"name": "getLimitCount",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint64"}
			]
		},
		{
			"name": "getLimits",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint64[]"}
			]
		}
	],
	"events": [
	],
	"data": [
		{"key":102,"name":"MAX_LIMIT_COUNT","type":"uint8"},
		{"key":103,"name":"SECONDS_IN_DAY","type":"uint32"},
		{"key":104,"name":"MAX_LIMIT_PERIOD","type":"uint32"}
	]
}
"#;
