extern crate abi_lib_dynamic;
extern crate serde_json;
extern crate tvm;

extern crate ed25519_dalek;
extern crate rand;
extern crate sha2;
extern crate pretty_assertions;

use abi_lib_dynamic::Contract;
use abi_lib_dynamic::token::{Tokenizer, Detokenizer};
use tvm::stack::SliceData;
use serde_json::Value;

use sha2::Sha512;
use ed25519_dalek::*;
use rand::rngs::OsRng;
use pretty_assertions::assert_eq;

const CREATE_LIMIT_PARAMS: &str = r#"
{
	"type": 18,
	"value": "1234567890",
	"meta": "x"
}"#;

const SEND_TRANSACTION_PARAMS: &str = r#"
{
	"recipient": "x0000000000000000000000000000000000000000000000000000000000000000",
	"value": 1234567890
}"#;

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

const SUBSCRIBE_PARAMS: &str = r#"
{
	"pubkey": "x0000000000000000000000000000000000000000000000000000000000000000",
	"to": "x0000000000000000000000000000000000000000000000000000000000000000",
	"value": 1234567890,
	"period": 1234567890
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

fn print_function_singnatures(abi: &str) {
	let contract = Contract::load(abi.as_bytes()).unwrap();

	let functions = contract.functions();

	for function in functions {
		//println!("{}", function.name);
		println!("{}", function.get_function_signature());
		let id = u32::from_be_bytes(function.get_function_id());
		println!("{:X?}\n", id);
	}
}


fn main() {
    let contract = Contract::load(SUBSCRIBE_CONTRACT_ABI.as_bytes()).unwrap();

	print_function_singnatures(PIGGY_BANK_CONTRACT_ABI);

	let function = contract.function("subscribe").unwrap();

    let v: Value = serde_json::from_str(SUBSCRIBE_PARAMS).unwrap();
    let tokens = Tokenizer::tokenize_all(&function.input_params(), &v).unwrap();

	let pair = Keypair::generate::<Sha512, _>(&mut OsRng::new().unwrap());

    let input = function.encode_input(&tokens, Some(&pair)).unwrap();

	let decoded_tokens = function.decode_input(SliceData::from(input)).unwrap();

	assert_eq!(tokens, decoded_tokens);

	for token in tokens {
    	println!("{}", token);
	}

	let json_output = Detokenizer::detokenize(&function.input_params(), &decoded_tokens).unwrap();

	println!("\n{}", json_output);
}