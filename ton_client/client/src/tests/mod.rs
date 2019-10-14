use ::{InteropContext, JsonResponse};
use ::{tc_json_request, InteropString};
use ::{tc_read_json_response, tc_destroy_json_response};
use serde_json::Value;
use log::{Metadata, Record, LevelFilter};
use {tc_create_context, tc_destroy_context};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

fn json_request(
    context: InteropContext,
    method_name: &str,
    params: Value,
) -> JsonResponse {
    unsafe {
        let params_json = if params.is_null() { String::new() } else { params.to_string() };
        let response_ptr = tc_json_request(
            context,
            InteropString::from(&method_name.to_string()),
            InteropString::from(&params_json),
        );
        let interop_response = tc_read_json_response(response_ptr);
        let response = interop_response.to_response();
        tc_destroy_json_response(response_ptr);
        response
    }
}


#[test]
fn test() {
    log::set_boxed_logger(Box::new(SimpleLogger))
        .map(|()| log::set_max_level(LevelFilter::Debug)).unwrap();
    unsafe {
        let context = tc_create_context();

        let version = json_request(context, "version", Value::Null);
        println!("result: {}", version.result_json.to_string());

        let _deployed = json_request(context, "setup",
            json!({"baseUrl": "http://0.0.0.0"}));

		let giver_abi: Value = serde_json::from_str(GIVER_ABI).unwrap();

		let result = json_request(context, "contracts.run",
            json!({
                "address": GIVER_ADDRESS,
                "abi": giver_abi,
                "functionName": "sendGrams",
                "input": &json!({
					"dest": format!("0x{}", WALLET_ADDRESS),
					"amount": 10_000_000_000u64
					}),
                "keyPair": {
                    "public": "d59bdd49a40013f6335753eb19b34b37f42ca25df8a44bd7388882ab57019dd1",
                    "secret": "4f255abd8da7dcf1fbc94ae2e2742d350621a99a4bd53592661f22ec25bf1d23"
                },
            }),
        );

		assert_eq!(result.error_json, "");

        let abi: Value = serde_json::from_str(WALLET_ABI).unwrap();
        let deployed = json_request(context, "contracts.deploy",
            json!({
                "abi": abi.clone(),
                "constructorParams": json!({}),
                "imageBase64": WALLET_CODE_BASE64,
                "keyPair": {
                    "public": "d59bdd49a40013f6335753eb19b34b37f42ca25df8a44bd7388882ab57019dd1",
                    "secret": "4f255abd8da7dcf1fbc94ae2e2742d350621a99a4bd53592661f22ec25bf1d23"
                },
            }),
        );

        assert_eq!("{\"address\":\"bbc3fbdb18379635480bbe3a0c520a2183e13b2cf1541a35a29446b72b331ed0\"}",
            deployed.result_json);

        let result = json_request(context, "contracts.run",
            json!({
                "address": WALLET_ADDRESS,
                "abi": abi.clone(),
                "functionName": "getVersion",
                "input": json!({}),
                "keyPair": {
                    "public": "d59bdd49a40013f6335753eb19b34b37f42ca25df8a44bd7388882ab57019dd1",
                    "secret": "4f255abd8da7dcf1fbc94ae2e2742d350621a99a4bd53592661f22ec25bf1d23"
                },
            }),
        );
        assert_eq!("{\"output\":{\"error\":\"-0x1\",\"version\":{\"major\":\"0x0\",\"minor\":\"0x1\"}}}",
            result.result_json);

        tc_destroy_context(context);
    }
}

const GIVER_ADDRESS: &str = "ce709b5bfca589eb621b5a5786d0b562761144ac48f59e0b0d35ad0973bcdb86";
const GIVER_ABI: &str = r#"
{
    "ABI version": 1,
	"data": [],
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

pub const WALLET_ADDRESS: &str = "bbc3fbdb18379635480bbe3a0c520a2183e13b2cf1541a35a29446b72b331ed0";
pub const WALLET_CODE_BASE64: &str = r#"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"#;
pub const WALLET_ABI: &str = r#"{
	"ABI version": 1,
	"data": [],
	"functions": [
		{
			"inputs": [
				{
					"name": "recipient",
					"type": "fixedbytes32"
				},
				{
					"name": "value",
					"type": "gram"
				}
			],
			"name": "sendTransaction",
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
					"type": "gram"
				},
				{
					"name": "meta",
					"type": "bytes"
				}
			],
			"name": "createLimit",
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
					"type": "gram"
				},
				{
					"name": "meta",
					"type": "bytes"
				}
			],
			"name": "changeLimitById",
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
							"type": "gram"
						},
						{
							"name": "type",
							"type": "uint8"
						},
						{
							"name": "meta",
							"type": "bytes"
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
			"name": "getLimitsEx",
			"outputs": [
				{
					"name": "list",
					"type": "tuple[]",
					"components": [
						{
							"name": "id",
							"type": "uint8"
						},
						{
							"name": "type",
							"type": "uint8"
						},
						{
							"name": "value",
							"type": "gram"
						},
						{
							"name": "meta",
							"type": "bytes"
						}
					]
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
			"inputs": [
				{
					"name": "address",
					"type": "fixedbytes32"
				}
			],
			"name": "setSubscriptionAccount",
			"outputs": []
		},
		{
			"inputs": [],
			"name": "getSubscriptionAccount",
			"outputs": [
				{
					"name": "address",
					"type": "fixedbytes32"
				}
			]
		}
	]
}
"#;
