use super::*;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use sha2::Sha512;
use tests_common::*;

#[test]
fn full_test_piggy_bank() {

    // connect to node
    init_node_connection();

	// generate key pair
    let mut csprng = OsRng::new().unwrap();
    let keypair = Keypair::generate::<Sha512, _>(&mut csprng);

    let now = std::time::Instant::now();

	// deploy wallet
    println!("Wallet contract deploying...\n");
    let wallet_address = deploy_contract_and_wait("LimitWallet.tvc", WALLET_ABI, "{}", &keypair, 0);
	println!("Wallet contract deployed. Account address {}\n", wallet_address);

	// deploy piggy bank
    println!("Piggy bank contract deploying...\n");
	let piggy_bank_address = deploy_contract_and_wait("Piggy.tvc", PIGGY_BANK_CONTRACT_ABI, PIGGY_BANK_CONSTRUCTOR_PARAMS, &keypair, 0);
	println!("Piggy bank contract deployed. Account address {}\n", piggy_bank_address);

    // get goal from piggy
    println!("Get goal from piggy...\n");
    //let (get_goal_answer, _) = call_contract_and_wait(piggy_bank_address.clone(), "getGoal", "{}".to_string(), PIGGY_BANK_CONTRACT_ABI, None);
    let get_goal_answer = local_contract_call(piggy_bank_address.clone(), "getGoal", "{}", PIGGY_BANK_CONTRACT_ABI, None);
    println!("piggy answer {}", get_goal_answer);

	// deploy subscription

    println!("Subscription contract deploying...\n");
	let subscription_constructor_params = format!("{{ \"wallet\" : \"{}\" }}", wallet_address);
	let subscripition_address = deploy_contract_and_wait("Subscription.tvc", SUBSCRIBE_CONTRACT_ABI, &subscription_constructor_params, &keypair, 0);
	println!("Subscription contract deployed. Account address {}\n", subscripition_address);


    // call setSubscriptionAccount in wallet
    println!("Adding subscription address to the wallet...\n");
	let set_subscription_params = format!("{{ \"addr\" : \"{}\" }}", subscripition_address);

	let _set_subscription_answer = call_contract(wallet_address, "setSubscriptionAccount", set_subscription_params, WALLET_ABI, Some(&keypair));

	println!("Subscription address added to the wallet.\n");

	// call subscribe in subscription
    println!("Adding subscription 1...\n");
    let subscr_id_str = hex::encode(&[0x11; 32]);
	let pubkey_str = hex::encode(keypair.public.as_bytes());
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"0x{}\", \"pubkey\" : \"0x{}\", \"to\": \"{}\", \"value\" : 123, \"period\" : 456 }}",
        subscr_id_str,
        &pubkey_str,
        piggy_bank_address,
    );

	call_contract(subscripition_address.clone(), "subscribe", subscribe_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
	println!("Subscription 1 added.\n");

    	// call subscribe in subscription
    println!("Adding subscription 2...\n");
    let subscr_id_str = hex::encode(&[0x22; 32]);
	let subscribe_params = format!(
        "{{ \"subscriptionId\" : \"0x{}\", \"pubkey\" : \"0x{}\", \"to\": \"{}\", \"value\" : 5000000000, \"period\" : 86400 }}",
        subscr_id_str,
        &pubkey_str,
        piggy_bank_address,
    );
	call_contract(subscripition_address.clone(), "subscribe", subscribe_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
	println!("Subscription 2 added.\n");

    println!("Call getSubscription with id {}\n", &subscr_id_str);
    let get_params = format!("{{ \"subscriptionId\" : \"0x{}\" }}", &subscr_id_str);
    let answer = local_contract_call(subscripition_address.clone(), "getSubscription", &get_params, SUBSCRIBE_CONTRACT_ABI, Some(&keypair));
    println!("getSubscription result:\n{}", answer);

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
				{"name":"wallet","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "getWallet",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"address"}
			]
		},
		{
			"name": "getSubscription",
			"inputs": [
				{"name":"subscriptionId","type":"uint256"}
			],
			"outputs": [
				{"components":[{"name":"pubkey","type":"uint256"},{"name":"to","type":"address"},{"name":"value","type":"uint64"},{"name":"period","type":"uint32"},{"name":"start","type":"uint32"},{"name":"status","type":"uint8"}],"name":"value0","type":"tuple"}
			]
		},
		{
			"name": "subscribe",
			"inputs": [
				{"name":"subscriptionId","type":"uint256"},
				{"name":"pubkey","type":"uint256"},
				{"name":"to","type":"address"},
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
		{"key":100,"name":"mywallet","type":"address"}
	]
}"#;

pub const PIGGY_BANK_CONTRACT_ABI: &str = r#"
{
	"ABI version": 1,
	"functions": [
		{
			"name": "constructor",
			"inputs": [
				{"name":"amount","type":"uint64"},
				{"name":"goal","type":"bytes"}
			],
			"outputs": [
			]
		},
		{
			"name": "transfer",
			"inputs": [
				{"name":"to","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "getGoal",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"bytes"}
			]
		},
		{
			"name": "getTargetAmount",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"uint64"}
			]
		}
	],
	"events": [
	],
	"data": [
		{"key":100,"name":"targetGoal","type":"bytes"},
		{"key":101,"name":"targetAmount","type":"uint64"}
	]
} "#;

// 123 Grams and "Some goal" as goal
const PIGGY_BANK_CONSTRUCTOR_PARAMS: &str = r#"
{
	"amount": 123,
	"goal": "536f6d6520676f616c"
}"#;


pub const WALLET_ABI: &str = r#"
{
	"ABI version": 1,
	"functions": [
		{
			"name": "sendTransaction",
			"inputs": [
				{"name":"dest","type":"address"},
				{"name":"value","type":"uint128"},
				{"name":"bounce","type":"bool"}
			],
			"outputs": [
			]
		},
		{
			"name": "setSubscriptionAccount",
			"inputs": [
				{"name":"addr","type":"address"}
			],
			"outputs": [
			]
		},
		{
			"name": "getSubscriptionAccount",
			"inputs": [
			],
			"outputs": [
				{"name":"value0","type":"address"}
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
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"events": [
	],
	"data": [
		{"key":101,"name":"subscription","type":"address"},
		{"key":100,"name":"owner","type":"uint256"}
	]
}
"#;
