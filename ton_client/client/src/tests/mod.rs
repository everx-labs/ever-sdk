use crypto::keys::account_decode;
use ::{InteropContext, JsonResponse};
use ::{tc_json_request, InteropString};
use ::{tc_read_json_response, tc_destroy_json_response};
use serde_json::Value;
use log::{Metadata, Record, LevelFilter};
use {tc_create_context, tc_destroy_context};
use ton_sdk::encode_base64;
use tvm::block::MsgAddressInt;

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
fn test_wallet_deploy() {
    log::set_boxed_logger(Box::new(SimpleLogger))
        .map(|()| log::set_max_level(LevelFilter::Debug)).unwrap();
    unsafe {
        let context = tc_create_context();

        let version = json_request(context, "version", Value::Null);
        println!("result: {}", version.result_json.to_string());

        let _deployed = json_request(context, "setup",
            json!({"baseUrl": "http://192.168.99.100"}));

		let keys = json_request(context, "crypto.ed25519.keypair", json!({}));

		assert_eq!(keys.error_json, "");

		let abi: Value = serde_json::from_str(WALLET_ABI).unwrap();
		let keys: Value = serde_json::from_str(&keys.result_json).unwrap();

		let address = json_request(context, "contracts.deploy.message",
            json!({
                "abi": abi.clone(),
                "constructorParams": json!({}),
                "imageBase64": WALLET_CODE_BASE64,
                "keyPair": keys,
                "workchainId": 0,
            }),
        );

		assert_eq!(address.error_json, "");

		let address = serde_json::from_str::<Value>(&address.result_json).unwrap()["address"].clone();
		let address = serde_json::from_value::<MsgAddressInt>(address).unwrap();

		let giver_abi: Value = serde_json::from_str(GIVER_ABI).unwrap();

		let result = json_request(context, "contracts.run",
            json!({
                "address": GIVER_ADDRESS,
                "abi": giver_abi,
                "functionName": "sendGrams",
                "input": &json!({
					"dest": format!("0x{:x}", address.get_address()),
					"amount": 10_000_000_000u64
					}),
            }),
        );

		assert_eq!(result.error_json, "");

		let wait_result = json_request(context, "queries.wait.for",
            json!({
                "table": "accounts".to_owned(),
                "filter": json!({
					"id": { "eq": address.to_string() },
					"storage": {
						"balance": {
							"Grams": { "gt": "0" }
						}
					}
				}).to_string(),
				"result": "id storage {balance {Grams}}".to_owned()
            }),
        );

		assert_eq!(wait_result.error_json, "");

        let deployed = json_request(context, "contracts.deploy",
            json!({
                "abi": abi.clone(),
                "constructorParams": json!({}),
                "imageBase64": WALLET_CODE_BASE64,
                "keyPair": keys,
                "workchainId": 0,
            }),
        );

        assert_eq!(format!("{{\"address\":\"{}\"}}", address), deployed.result_json);

        let result = json_request(context, "contracts.run",
            json!({
                "address": address.to_string(),
                "abi": abi.clone(),
                "functionName": "getLimitCount",
                "input": json!({}),
                "keyPair": keys,
            }),
        );
        assert_eq!("{\"output\":{\"value0\":\"0x0\"}}", result.result_json);

        tc_destroy_context(context);
    }
}

const GIVER_ADDRESS: &str = "a46af093b38fcae390e9af5104a93e22e82c29bcb35bf88160e4478417028884";
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

pub const WALLET_CODE_BASE64: &str = r#"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"#;
pub const WALLET_ABI: &str = r#"{
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

#[test]
fn test_address_parsing() {
    let short = "fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let full_std = "-1:fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let base64 = "kf/8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15+KsQHFLbKSMiYIny";
    let base64_url = "kf_8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15-KsQHFLbKSMiYIny";

    let address = tvm::block::MsgAddressInt::with_standart(None, -1, hex::decode(short).unwrap().into()).unwrap();
    let wc0_address = tvm::block::MsgAddressInt::with_standart(None, 0, hex::decode(short).unwrap().into()).unwrap();

    assert_eq!(wc0_address, account_decode(short).expect("Couldn't parse short address"));
    assert_eq!(address, account_decode(full_std).expect("Couldn't parse full_std address"));
    assert_eq!(address, account_decode(base64).expect("Couldn't parse base64 address"));
    assert_eq!(address, account_decode(base64_url).expect("Couldn't parse base64_url address"));

    assert_eq!(encode_base64(&address, true, true, false).unwrap(), base64);
    assert_eq!(encode_base64(&address, true, true, true ).unwrap(), base64_url);
}

#[test]
fn test_print_base64_address_from_hex() {
    let hex_address = "0:9f2bc8a81da52c6b8cb1878352120f21e254138fff0b897f44fb6ff2b8cae256";

    let address = account_decode(hex_address).unwrap();

    println!("{}", encode_base64(&address, false, false, false).unwrap());
}
