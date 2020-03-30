/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::crypto::keys::{account_decode, account_encode_ex, AccountAddressType, Base64AddressParams};
use super::InteropContext;
use super::{tc_json_request, InteropString};
use super::{tc_read_json_response, tc_destroy_json_response};
use serde_json::{Value, Map};
use log::{Metadata, Record, LevelFilter};
use crate::{tc_create_context, tc_destroy_context};
use ton_block::MsgAddressInt;
use std::str::FromStr;

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

struct TestClient {
    context: InteropContext,
}

impl TestClient {
    fn new() -> Self {
        let _ = log::set_boxed_logger(Box::new(SimpleLogger))
            .map(|()| log::set_max_level(LevelFilter::Debug));

        let context: InteropContext;
        unsafe {
            context = tc_create_context()
        }
        Self { context }
    }

    fn request(
        &self,
        method_name: &str,
        params: Value,
    ) -> Result<String, String> {
        unsafe {
            let params_json = if params.is_null() { String::new() } else { params.to_string() };
            let response_ptr = tc_json_request(
                self.context,
                InteropString::from(&method_name.to_string()),
                InteropString::from(&params_json),
            );
            let interop_response = tc_read_json_response(response_ptr);
            let response = interop_response.to_response();
            tc_destroy_json_response(response_ptr);
            if response.error_json.is_empty() {
                Ok(response.result_json)
            } else {
                Err(response.error_json)
            }
        }
    }
}

impl Drop for TestClient {
    fn drop(&mut self) {
        unsafe {
            tc_destroy_context(self.context)
        }
    }
}


fn parse_object(s: Result<String, String>) -> Map<String, Value> {
    if let Value::Object(m) = serde_json::from_str(s.unwrap().as_str()).unwrap() {
        return m.clone();
    }
    panic!("Object expected");
}

fn parse_string(r: Result<String, String>) -> String {
    if let Value::String(s) = serde_json::from_str(r.unwrap().as_str()).unwrap() {
        return s.clone();
    }
    panic!("String expected");
}

fn get_map_string(m: &Map<String, Value>, f: &str) -> String {
    if let Value::String(s) = m.get(f).unwrap() {
        return s.clone();
    }
    panic!("Field not fount");
}

#[test]
fn test_tg_mnemonic() {
    let client = TestClient::new();
    let crc16 = client.request("crypto.ton_crc16", json!({
        "hex": "0123456789abcdef"
    })).unwrap();
    assert_eq!(crc16, "43349");

    let keys = parse_object(client.request(
        "crypto.mnemonic.derive.sign.keys",
        json!({
            "phrase": "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry teach mechanic rain float vicious solution assume hedgehog rail sort chuckle"
        }),
    ));
    let ton_public = parse_string(client.request(
        "crypto.ton_public_key_string",
        Value::String(get_map_string(&keys, "public")),
    ));
    assert_eq!(ton_public, "PubDdJkMyss2qHywFuVP1vzww0TpsLxnRNnbifTCcu-XEgW0");

    let words = parse_string(client.request("crypto.mnemonic.words", json!({
    })));
    assert_eq!(words.split(" ").count(), 2048);

    let phrase = parse_string(client.request("crypto.mnemonic.from.random", json!({
    })));
    assert_eq!(phrase.split(" ").count(), 24);

    let entropy = "2199ebe996f14d9e4e2595113ad1e6276bd05e2e147e16c8ab8ad5d47d13b44fcf";
    let mnemonic = parse_string(client.request("crypto.mnemonic.from.entropy", json!({
        "entropy": json!({
            "hex": entropy
        }),
    })));
    let public = get_map_string(&parse_object(client.request(
        "crypto.mnemonic.derive.sign.keys",
        json!({
            "phrase": mnemonic
        }),
    )), "public");
    let ton_public = parse_string(client.request(
        "crypto.ton_public_key_string",
        Value::String(public),
    ));
    assert_eq!(ton_public, "PuYGEX9Zreg-CX4Psz5dKehzW9qCs794oBVUKqqFO7aWAOTD");
//    let ton_phrase = "shove often foil innocent soft slim pioneer day uncle drop nephew soccer worry renew public hand word nut again dry first delay first maple";
    let is_valid = client.request(
        "crypto.mnemonic.verify",
        json!({
            "phrase": "unit follow zone decline glare flower crisp vocal adapt magic much mesh cherry teach mechanic rain float vicious solution assume hedgehog rail sort chuckle"
        }),
    ).unwrap();
    assert_eq!(is_valid, "true");
    let is_valid = client.request(
        "crypto.mnemonic.verify",
        json!({
            "phrase": "unit follow"
        }),
    ).unwrap();
    assert_eq!(is_valid, "false");
    let is_valid = client.request(
        "crypto.mnemonic.verify",
        json!({
            "phrase": "unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit unit"
        }),
    ).unwrap();
    assert_eq!(is_valid, "false");
}

#[test]
fn test_wallet_deploy() {
    let client = TestClient::new();
    let version = client.request("version", Value::Null).unwrap();
    println!("result: {}", version.to_string());

    let _deployed = client.request("setup",
        json!({"baseUrl": "http://localhost"})).unwrap();

    let keys = client.request("crypto.ed25519.keypair", json!({})).unwrap();

    let abi: Value = serde_json::from_str(WALLET_ABI).unwrap();
    let keys: Value = serde_json::from_str(&keys).unwrap();

    let address = client.request("contracts.deploy.message",
        json!({
                "abi": abi.clone(),
                "constructorParams": json!({}),
                "imageBase64": WALLET_CODE_BASE64,
                "keyPair": keys,
                "workchainId": 0,
            }),
    ).unwrap();

    let address = serde_json::from_str::<Value>(&address).unwrap()["address"].clone();
    let address = MsgAddressInt::from_str(address.as_str().unwrap()).unwrap();

    let giver_abi: Value = serde_json::from_str(GIVER_ABI).unwrap();

    let _ = client.request("contracts.run",
        json!({
                "address": GIVER_ADDRESS,
                "abi": giver_abi,
                "functionName": "sendGrams",
                "input": &json!({
					"dest": address.to_string(),
					"amount": 10_000_000_000u64
					}),
            }),
    ).unwrap();

    let _ = client.request("queries.wait.for",
        json!({
                "table": "accounts".to_owned(),
                "filter": json!({
					"id": { "eq": address.to_string() },
					"balance": { "gt": "0" }
				}).to_string(),
				"result": "id balance".to_owned()
            }),
    ).unwrap();

    let deployed = client.request("contracts.deploy",
        json!({
                "abi": abi.clone(),
                "constructorParams": json!({}),
                "imageBase64": WALLET_CODE_BASE64,
                "keyPair": keys,
                "workchainId": 0,
            }),
    ).unwrap();

    assert_eq!(format!("{{\"address\":\"{}\",\"alreadyDeployed\":false}}", address), deployed);

    let result = client.request("contracts.run",
        json!({
                "address": address.to_string(),
                "abi": abi.clone(),
                "functionName": "createOperationLimit",
                "input": json!({
					"value": 123
				}),
                "keyPair": keys,
            }),
    ).unwrap();
    assert_eq!("{\"output\":{\"value0\":\"0x0\"}}", result);
}

const GIVER_ADDRESS: &str = "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94";
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
				{"name":"dest","type":"address"},
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

#[test]
fn test_address_parsing() {
    let short = "fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let full_std = "-1:fcb91a3a3816d0f7b8c2c76108b8a9bc5a6b7a55bd79f8ab101c52db29232260";
    let base64 = "kf/8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15+KsQHFLbKSMiYIny";
    let base64_url = "kf_8uRo6OBbQ97jCx2EIuKm8Wmt6Vb15-KsQHFLbKSMiYIny";

    let address = ton_block::MsgAddressInt::with_standart(None, -1, hex::decode(short).unwrap().into()).unwrap();
    let wc0_address = ton_block::MsgAddressInt::with_standart(None, 0, hex::decode(short).unwrap().into()).unwrap();

    assert_eq!(wc0_address, account_decode(short).expect("Couldn't parse short address"));
    assert_eq!(address, account_decode(full_std).expect("Couldn't parse full_std address"));
    assert_eq!(address, account_decode(base64).expect("Couldn't parse base64 address"));
    assert_eq!(address, account_decode(base64_url).expect("Couldn't parse base64_url address"));

    assert_eq!(account_encode_ex(
            &address,
            AccountAddressType::Base64,
            Some(Base64AddressParams {
                bounce: true,
                test: true,
                url: false
            })).unwrap(),
        base64);
    assert_eq!(account_encode_ex(
            &address,
            AccountAddressType::Base64,
            Some(Base64AddressParams {
                bounce: true,
                test: true,
                url: true
        })).unwrap(),
        base64_url);
}
