/*
* Copyright 2018-2021 EverX Labs Ltd.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/
use crate::crypto::{KeyPair, SigningBoxHandle, EncryptionBoxHandle, RegisteredSigningBox,
    RegisteredEncryptionBox, EncryptionBoxInfo,boxes::encryption_box::EncryptionBox};
use crate::encoding::decode_abi_number;
use std::sync::Arc;
use crate::tests::TestClient;
use crate::client::ParamsOfAppRequest;
use serde_json::Value;
use crate::ClientContext;
use crate::error::ClientResult;
use crate::json_interface::crypto::*;
use crate::json_interface::interop::ResponseType;
//use super::*;


pub const SUPPORTED_INTERFACES: &[&str] = &[
    "f6927c0d4bdb69e1b52d27f018d156ff04152f00558042ff674f0fec32e4369d", // echo
    "8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3", // terminal
    "c13024e101c95e71afb1f5fa6d72f633d51e721de0320d73dfd6121a54e4d40a", // signing box input,
    "5b5f76b54d976d72f1ada3063d1af2e5352edaf1ba86b3b311170d4d81056d61", // encryption box input
];

pub const MY_TEST_PUBKEY: &str = "0xb7cb10668eb106f91293014f6f47657f2f6b1b47332b4c865a874905271e95b3";

pub const ECHO_ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "echo",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"request","type":"bytes"}
			],
			"outputs": [
				{"name":"response","type":"bytes"}
			]
		}
	],
	"data": [],
	"events": []
}
"#;

pub const TERMINAL_ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "print",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"message","type":"bytes"}
			],
			"outputs": []
        },
        {
			"name": "inputInt",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"}
			],
			"outputs": [
				{"name":"value","type":"int256"}
			]
		},
        {
			"name": "input",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"},
				{"name":"multiline","type":"bool"}
			],
			"outputs": [
				{"name":"value","type":"bytes"}
			]
		}
	],
	"data": [],
	"events": []
}
"#;

pub const SIGNING_BOX_ABI: &str = r#"
{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "get",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"},
				{"name":"possiblePublicKeys","type":"uint256[]"}
			],
			"outputs": [
				{"name":"handle","type":"uint32"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}
"#;

pub const SIGNING_BOX_ABI_2_2: &str = r#"
{
	"ABI version": 2,
	"version": "2.2",
	"header": ["time"],
	"functions": [
		{
			"name": "get",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"string"},
				{"name":"possiblePublicKeys","type":"uint256[]"}
			],
			"outputs": [
				{"name":"handle","type":"uint32"}
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
	"data": [
	],
	"events": [
	],
	"fields": [
		{"name":"_pubkey","type":"uint256"},
		{"name":"_timestamp","type":"uint64"},
		{"name":"_constructorFlag","type":"bool"}
	]
}
"#;


pub const ENCRYPTION_BOX_ABI: &str = r#"{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "getNaclBox",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"prompt","type":"bytes"},
				{"name":"nonce","type":"bytes"},
				{"name":"theirPubkey","type":"uint256"}
			],
			"outputs": [
				{"name":"handle","type":"uint32"}
			]
		},
		{
			"name": "remove",
			"inputs": [
				{"name":"answerId","type":"uint32"},
				{"name":"handle","type":"uint32"}
			],
			"outputs": [
				{"name":"removed","type":"bool"}
			]
		},
		{
			"name": "getSupportedAlgorithms",
			"inputs": [
				{"name":"answerId","type":"uint32"}
			],
			"outputs": [
				{"name":"names","type":"bytes[]"}
			]
		}
	],
	"data": [
	],
	"events": [
	]
}"#;

pub(crate) struct EncryptionBoxInput {
    client: Arc<TestClient>,
}

impl EncryptionBoxInput {
    pub(crate) async fn new(client: Arc<TestClient>) -> Self {
        Self{  client }
    }

    pub async fn call(&self, func: &str, args: &Value) -> (u32, Value) {
        match func {
            "getNaclBox" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                let nonce = args["nonce"].as_str().unwrap().to_owned();
                let their_key = args["theirPubkey"].as_str().unwrap().to_owned();
                let client_copy = Arc::clone(&self.client);
                let context = self.client.context();
                let callback = move |params, response_type| {
                    let client = Arc::clone(&client_copy);
                    let context = Arc::clone(&context);
                    let nonce = nonce.clone();
                    let their_key = their_key.clone();
                    async move {
                        match response_type {
                            ResponseType::AppRequest => {
                                tokio::spawn(async move {
                                    let request: ParamsOfAppRequest = serde_json::from_value(params).unwrap();
                                    let result = Self::process_call(
                                        context,
                                        NaclBoxEncryption::new(nonce, their_key),
                                        serde_json::from_value(request.request_data).unwrap()
                                    ).await;
                                    client.resolve_app_request(request.app_request_id, result).await;
                                });
                            },
                            _ => panic!("Wrong response type"),
                        }
                    }
                };
                let box_handle: EncryptionBoxHandle = self.client.request_async_callback(
                    "crypto.register_encryption_box",
                    json!({}),
                    callback,
                )
                .await
                .map(|x: RegisteredEncryptionBox| x.handle).unwrap_or(EncryptionBoxHandle(0));

                ( answer_id, json!({ "handle": box_handle.clone() }) )
            },
            "getSupportedInterfaces" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                ( answer_id, json!({ "names": vec![hex::encode("NaclBox")] }) )
            },
            "remove" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                ( answer_id, json!({ "removed": true }) )
            },
            _ => panic!("interface function not found"),
        }
    }

    async fn process_call(
        context: Arc<ClientContext>,
        enbox: impl EncryptionBox + 'static,
        params: ParamsOfAppEncryptionBox,
    ) -> ResultOfAppEncryptionBox {
        match params {
            ParamsOfAppEncryptionBox::GetInfo => {
                ResultOfAppEncryptionBox::GetInfo { info: enbox.get_info(context).await.unwrap() }
            },
            ParamsOfAppEncryptionBox::Encrypt {data} => {
                ResultOfAppEncryptionBox::Encrypt { data: enbox.encrypt(context, &data).await.unwrap() }
            },
            ParamsOfAppEncryptionBox::Decrypt {data} => {
                ResultOfAppEncryptionBox::Decrypt { data: enbox.decrypt(context, &data).await.unwrap() }
            },
        }
    }
}

struct NaclBoxEncryption {
    nonce: String,
    their_key: String,
}

impl NaclBoxEncryption {
    fn new(nonce: String, their_key: String) -> Self {
        Self{ nonce, their_key }
    }
}

#[async_trait::async_trait]
impl EncryptionBox for NaclBoxEncryption {
    async fn get_info(&self, _context: Arc<ClientContext>) -> ClientResult<EncryptionBoxInfo> {
        // emulate getnifo
        Ok(EncryptionBoxInfo {
            hdpath: Some(format!("m/44'/396'/0'/0/1")),
            algorithm: Some(format!("NaclBox")),
            options: Some(json!({"nonce": self.nonce, "theirPubkey": self.their_key })),
            public: Some(json!({"key": MY_TEST_PUBKEY})),
        })
    }

    async fn encrypt(&self, _context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        // emulate encryption
        Ok(hex::encode(data))
    }

    async fn decrypt(&self, _context: Arc<ClientContext>, data: &String) -> ClientResult<String> {
        // emulate decryption
        Ok(String::from_utf8(hex::decode(data).unwrap()).unwrap())
    }
}

pub(crate) struct SingingBoxInput {
    box_handle: SigningBoxHandle,
}

impl SingingBoxInput {
    pub(crate) async fn new(client: Arc<TestClient>, keys: KeyPair) -> Self {
        let box_handle = client.request_async::<_, RegisteredSigningBox>(
            "crypto.get_signing_box",
            keys,
        ).await.map(|x| x.handle).unwrap_or(SigningBoxHandle(0));
        Self{  box_handle }
    }

    pub fn call(&self, func: &str, args: &Value) -> (u32, Value) {
        match func {
            "get" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                ( answer_id, json!({ "handle": self.box_handle.clone() }) )
            },
            _ => panic!("interface function not found"),
        }
    }
}

pub struct Echo {}
impl Echo {
    pub fn new() -> Self {
        Self{}
    }
    pub fn call(&self, func: &str, args: &Value) -> (u32, Value) {
        match func {
            "echo" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                let request_vec = hex::decode(args["request"].as_str().unwrap()).unwrap();
                let request = std::str::from_utf8(&request_vec).unwrap();
                ( answer_id, json!({ "response": hex::encode(request.as_bytes()) }) )
            },
            _ => panic!("interface function not found"),
        }
    }
}

pub struct Terminal {
    pub messages: Vec<String>,
}
impl Terminal {
    pub fn new(messages: Vec<String>) -> Self {
        Self { messages }
    }
    fn print(&mut self, answer_id: u32, message: &str) -> (u32, Value) {
        assert!(
            self.messages.len() > 0,
            "Unexpected terminal message received: \"{}\"",
            message
        );
        assert_eq!(
            self.messages.remove(0),
            message,
            "Terminal message assert failed"
        );
        ( answer_id, json!({ }) )
    }

    pub fn call(&mut self, func: &str, args: &Value) -> (u32, Value) {
        match func {
            "print" => {
                let answer_id = decode_abi_number::<u32>(args["answerId"].as_str().unwrap()).unwrap();
                let message = hex::decode(args["message"].as_str().unwrap()).unwrap();
                let message = std::str::from_utf8(&message).unwrap();
                self.print(answer_id, message)
            },
            "inputInt" => {
                let answer_id = decode_abi_number::<u32>(args["answerId"].as_str().unwrap()).unwrap();
                let prompt = hex::decode(args["prompt"].as_str().unwrap()).unwrap();
                let prompt = std::str::from_utf8(&prompt).unwrap();
                let _ = self.print(answer_id, prompt);
                // use test return value here.
                (answer_id, json!({"value": 1}))
            },
            "input" => {
                let answer_id = decode_abi_number::<u32>(args["answerId"].as_str().unwrap()).unwrap();
                let prompt = hex::decode(args["prompt"].as_str().unwrap()).unwrap();
                let message = std::str::from_utf8(&prompt).unwrap();
                let _ = args["multiline"].as_bool().unwrap();
                self.print(answer_id, message);
                let value = "testinput";
                (answer_id, json!({ "value": hex::encode(value.as_bytes()) }) )
            },
            _ => panic!("interface function not found"),
        }
    }
}
