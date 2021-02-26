/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::abi::{CallSet, DeploySet, ParamsOfEncodeMessage, Signer, Abi,
    ParamsOfDecodeMessageBody, DecodedMessageBody, ResultOfEncodeInternalMessage, ParamsOfEncodeInternalMessage};
use crate::boc::{ParamsOfParse, ResultOfParse};
use crate::client::ParamsOfAppRequest;
use crate::crypto::KeyPair;
use crate::encoding::decode_abi_number;
use crate::json_interface::debot::*;
use crate::json_interface::interop::ResponseType;
use crate::net::ResultOfQueryCollection;
use crate::tests::{TEST_DEBOT, TEST_DEBOT_TARGET, TestClient};
use crate::tvm::{ParamsOfRunTvm, ResultOfRunTvm};
use futures::future::{BoxFuture, FutureExt};
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use crate::net::ParamsOfQueryCollection;
use super::*;

lazy_static!(
    static ref DEBOT: Mutex<Option<DebotData>> = Mutex::new(None);
);

const TEST_DEBOT2: &'static str = "testDebot2";
const TEST_DEBOT3: &'static str = "testDebot3";
const TEST_DEBOT4: &'static str = "testDebot4";

const SUPPORTED_INTERFACES: &[&str] = &[
    "f6927c0d4bdb69e1b52d27f018d156ff04152f00558042ff674f0fec32e4369d", // echo
    "8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3", // terminal
];
const ECHO_ABI: &str = r#"
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

const TERMINAL_ABI: &str = r#"
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
		}
	],
	"data": [],
	"events": []
}
"#;

struct Echo {}
impl Echo {
    fn new() -> Self {
        Self{}
    }
    fn call(&self, func: &str, args: &JsonValue) -> (u32, JsonValue) {
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

struct Terminal {
    messages: Vec<String>,
}
impl Terminal {
    fn new(messages: Vec<String>) -> Self {
        Self { messages }
    }
    fn print(&mut self, answer_id: u32, message: &str) -> (u32, JsonValue) {
        assert!(
            self.messages.len() > 0,
            format!("Terminal.messages vector is empty but must contains \"{}\"", message)
        );
        assert_eq!(self.messages.remove(0), message, "Terminal.print assert");
        ( answer_id, json!({ }) )
    }

    fn call(&mut self, func: &str, args: &JsonValue) -> (u32, JsonValue) {
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
            }
            _ => panic!("interface function not found"),
        }
    }
}

struct TestBrowser {}

#[derive(Default, Deserialize)]
#[serde(default)]
struct DebotStep {
    pub choice: u8,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub invokes: Vec<Vec<DebotStep>>,
}

#[derive(Default)]
struct CurrentStepData {
    pub available_actions: Vec<DebotAction>,
    pub outputs: Vec<String>,
    pub step: DebotStep,
}

struct BrowserData {
    pub current: Mutex<CurrentStepData>,
    pub next: Mutex<Vec<DebotStep>>,
    pub keys: KeyPair,
    pub address: String,
    pub client: Arc<TestClient>,
    pub finished: AtomicBool,
    pub switch_started: AtomicBool,
    pub msg_queue: Mutex<VecDeque<String>>,
    pub terminal: Mutex<Terminal>,
    pub echo: Echo,
    pub bots: Mutex<HashMap<String, RegisteredDebot>>,
}

impl TestBrowser {
    async fn fetch_debot(client: Arc<TestClient>, state: Arc<BrowserData>, address: String, start_function: &str) -> RegisteredDebot {
        let state_copy = state.clone();
        let client_copy = client.clone();
        let callback = move |params, response_type| {
            log::debug!("received from debot: {:#}", params);
            let client = client_copy.clone();
            let state = state_copy.clone();
            async move {
                match response_type {
                    ResponseType::AppNotify => {
                        Self::process_notification(&state, serde_json::from_value(params).unwrap()).await;
                    },
                    ResponseType::AppRequest => {
                        tokio::spawn(async move {
                            let request: ParamsOfAppRequest = serde_json::from_value(params).unwrap();
                            let result = Self::process_call(
                                client.clone(),
                                &state,
                                serde_json::from_value(request.request_data).unwrap()
                            ).await;
                            client.resolve_app_request(request.app_request_id, result).await;
                        });
                    },
                    _ => panic!("Wrong response type"),
                }
            }
        };

        let handle: RegisteredDebot = client.request_async_callback(
            start_function,
            ParamsOfStart { address: address.clone() },
            callback
        ).await.unwrap();

        let handle_copy = RegisteredDebot {
            debot_handle: handle.debot_handle.clone(),
            debot_abi: handle.debot_abi.clone(),
        };
        state.bots.lock().await.insert(address.clone(), handle_copy);
        handle
    }

    pub async fn execute_from_state(client: Arc<TestClient>, state: Arc<BrowserData>, start_function: &str) {
        let handle = Self::fetch_debot(client.clone(), state.clone(), state.address.clone(), start_function).await;

        while !state.finished.load(Ordering::Relaxed) {
            Self::handle_message_queue(client.clone(), state.clone()).await;

            let available_steps_count = state.current.lock().await.available_actions.len();
            if available_steps_count == 0 {
                break;
            }
            let action = {
                let mut step = state.current.lock().await;
                step.step = state.next.lock().await.remove(0);
                step.outputs.clear();
                step.available_actions[step.step.choice as usize - 1].clone()
            };
            log::info!("Executing action: {:#?}", action);
            let _: () = client.request_async(
                "debot.execute",
                ParamsOfExecute {
                    debot_handle: handle.debot_handle.clone(),
                    action
                }).await.unwrap();

            let step = state.current.lock().await;
            assert_eq!(step.outputs.len(), step.step.outputs.len());
            step.outputs.iter().zip(step.step.outputs.iter())
            .for_each(|outs| {
                match outs.1.find("{}") {
                    Some(pos) => assert_eq!(
                        outs.0.get(..pos).unwrap(),
                        outs.1.get(..pos).unwrap(),
                    ),
                    None => assert_eq!(outs.0, outs.1),
                };
            });
            assert_eq!(step.step.inputs.len(), 0);
            assert_eq!(step.step.invokes.len(), 0);

            if step.available_actions.len() == 0 { break; }
        }

        assert_eq!(state.next.lock().await.len(), 0);
        assert_eq!(state.terminal.lock().await.messages.len(), 0);
    }

    pub async fn execute(
        client: Arc<TestClient>,
        address: String,
        keys: KeyPair,
        steps: Vec<DebotStep>,
        terminal_outputs: Vec<String>,
    ) {
        Self::execute_with_func(client, address, keys, steps, terminal_outputs, "debot.start").await;
    }

    pub async fn execute_with_func(
        client: Arc<TestClient>,
        address: String,
        keys: KeyPair,
        steps: Vec<DebotStep>,
        terminal_outputs: Vec<String>,
        entry_function: &str,
    ) {
        let state = Arc::new(BrowserData {
            current: Mutex::new(Default::default()),
            next: Mutex::new(steps),
            client: client.clone(),
            keys,
            address: address.clone(),
            finished: AtomicBool::new(false),
            switch_started: AtomicBool::new(false),
            msg_queue: Mutex::new(Default::default()),
            terminal: Mutex::new(Terminal::new(terminal_outputs)),
            echo: Echo::new(),
            bots: Mutex::new(HashMap::new()),
        });

        Self::execute_from_state(client, state, entry_function).await
    }

    async fn process_notification(state: &BrowserData, params: ParamsOfAppDebotBrowser) {
        match params {
            ParamsOfAppDebotBrowser::Log{ msg } => {
                state.current.lock().await.outputs.push(msg);
            },
            ParamsOfAppDebotBrowser::Switch { context_id } => {
                assert_eq!(state.switch_started.swap(true, Ordering::Relaxed), false);
                if context_id == STATE_EXIT {
                    state.finished.store(true, Ordering::Relaxed);
                }
                state.current.lock().await.available_actions.clear();
            },
            ParamsOfAppDebotBrowser::SwitchCompleted => {
                assert_eq!(state.switch_started.swap(false, Ordering::Relaxed), true);
            },
            ParamsOfAppDebotBrowser::ShowAction { action } => {
                state.current.lock().await.available_actions.push(action);
            },
            ParamsOfAppDebotBrowser::Send { message } => {
                state.msg_queue.lock().await.push_back(message);
            },
            _ => panic!("invalid notification {:#?}", params)
        }
    }

    fn call_execute_boxed(
        client: Arc<TestClient>, state: Arc<BrowserData>, start_function: &'static str
    ) -> BoxFuture<'static, ()> {
        Self::execute_from_state(client, state, start_function).boxed()
    }

    async fn process_call(client: Arc<TestClient>, state: &BrowserData, params: ParamsOfAppDebotBrowser) -> ResultOfAppDebotBrowser {
        match params {
            ParamsOfAppDebotBrowser::Input { prompt: _ } => {
                let value = state.current.lock().await.step.inputs.remove(0);
                ResultOfAppDebotBrowser::Input { value: value.to_owned() }
            },
            ParamsOfAppDebotBrowser::GetSigningBox => {
                let signing_box: crate::crypto::RegisteredSigningBox = client.request_async(
                    "crypto.get_signing_box",
                    state.keys.clone()
                ).await.unwrap();

                ResultOfAppDebotBrowser::GetSigningBox { signing_box: signing_box.handle }
            },
            ParamsOfAppDebotBrowser::InvokeDebot { action, debot_addr } => {
                let mut steps = state.current.lock().await.step.invokes.remove(0);
                steps[0].choice = 1;
                let current = CurrentStepData {
                    available_actions: vec![action],
                    ..Default::default()
                };

                let state = Arc::new(BrowserData {
                    current: Mutex::new(current),
                    next: Mutex::new(steps),
                    client: client.clone(),
                    keys: state.keys.clone(),
                    address: debot_addr,
                    finished: AtomicBool::new(false),
                    switch_started: AtomicBool::new(false),
                    msg_queue: Mutex::new(Default::default()),
                    terminal: Mutex::new(Terminal::new(vec![])),
                    echo: Echo::new(),
                    bots: Mutex::new(HashMap::new()),
                });
                Self::call_execute_boxed(client, state, "debot.fetch").await;
                ResultOfAppDebotBrowser::InvokeDebot
            },
            _ => panic!("invalid call {:#?}", params)
        }
    }

    async fn handle_message_queue(
        client: Arc<TestClient>,
        state: Arc<BrowserData>,
    ) {
        let mut msg_opt = state.msg_queue.lock().await.pop_front();
        while let Some(msg) = msg_opt {
            let parsed: ResultOfParse = client.request_async(
                "boc.parse_message",
                ParamsOfParse { boc: msg.clone() },
            ).await.unwrap();

            let body = parsed.parsed["body"].as_str().unwrap().to_owned();
            let dest_addr = parsed.parsed["dst"].as_str().unwrap();
            let src_addr = parsed.parsed["src"].as_str().unwrap();
            let wc_and_addr: Vec<_> = dest_addr.split(':').collect();
            let interface_id = wc_and_addr[1];
            let wc = i8::from_str_radix(wc_and_addr[0], 10).unwrap();

            if wc == DEBOT_WC {
                assert_eq!(SUPPORTED_INTERFACES.contains(&interface_id), true);
                let abi = if SUPPORTED_INTERFACES[0] == interface_id {
                    Abi::Json(ECHO_ABI.to_owned())
                } else if SUPPORTED_INTERFACES[1] == interface_id {
                    Abi::Json(TERMINAL_ABI.to_owned())
                } else {
                    panic!("unsupported interface");
                };
                let decoded: DecodedMessageBody = client.request_async(
                    "abi.decode_message_body",
                    ParamsOfDecodeMessageBody { abi, body, is_internal: true },
                ).await.unwrap();
                let (func, args) = (decoded.name, decoded.value.unwrap());
                log::info!("request: {} ({})", func, args);
                let (func_id, return_args) =
                if SUPPORTED_INTERFACES[0] == interface_id {
                    state.echo.call(&func, &args)
                } else {
                    state.terminal.lock().await.call(&func, &args)
                };
                log::info!("response: {} ({})", func_id, return_args);

                let call_set = match func_id {
                    0 => None,
                    _ => CallSet::some_with_function_and_input(&format!("0x{:x}", func_id), return_args),
                };
                let bots = state.bots.lock().await;
                let handle = bots.get(src_addr).unwrap();
                let message = encode_internal_message(client.clone(), &handle.debot_abi, src_addr.to_owned(), call_set).await;
                debot_send(client.clone(), handle.debot_handle.clone(), message).await;

            } else {
                let debot_fetched = state.bots.lock().await.get(dest_addr).is_some();
                if !debot_fetched {
                    TestBrowser::fetch_debot(
                        client.clone(),
                        state.clone(),
                        dest_addr.to_owned(),
                        "debot.fetch",
                    ).await;

                }
                let debot_handle = state.bots.lock().await.get(dest_addr).unwrap().debot_handle.clone();
                debot_send(client.clone(), debot_handle, msg).await;
            }

            msg_opt = state.msg_queue.lock().await.pop_front();
        }
    }
}

async fn encode_internal_message(client: Arc<TestClient>, abi: &str, addr: String, call_set: Option<CallSet>) -> String {
    let r: ResultOfEncodeInternalMessage = client.request_async(
        "abi.encode_internal_message",
        ParamsOfEncodeInternalMessage {
            abi: Abi::Contract(serde_json::from_str(abi).unwrap()),
            address: Some(addr),
            deploy_set: None,
            call_set,
            value: "1000000000000000".to_owned(),
            bounce: None,
            enable_ihr: None,
        }
    ).await.unwrap();
    r.message
}

async fn debot_send(client: Arc<TestClient>, debot_handle: DebotHandle, message: String) {
    let _result: () = client.request_async(
        "debot.send",
        ParamsOfSend {
            debot_handle,
            message,
        }
    ).await.unwrap();
}

#[derive(Clone)]
struct DebotData {
    debot_addr: String,
    target_addr: String,
    keys: KeyPair,
}

async fn init_debot(client: Arc<TestClient>) -> DebotData {
    let mut debot = DEBOT.lock().await;

    if let Some(data) = &*debot {
        return data.clone();
    }

    let keys = client.generate_sign_keys();

    let target_abi = TestClient::abi(TEST_DEBOT_TARGET, Some(2));
    let debot_abi = TestClient::abi(TEST_DEBOT, Some(2));

    let target_deploy_params = ParamsOfEncodeMessage {
        abi: target_abi.clone(),
        deploy_set: Some(DeploySet {
            tvc: TestClient::tvc(TEST_DEBOT_TARGET, Some(2)),
            ..Default::default()
        }),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };

    let target_addr = client.encode_message(target_deploy_params.clone()).await.unwrap().address;


    let target_future = client.deploy_with_giver_async(
        ParamsOfEncodeMessage {
            abi: target_abi.clone(),
            deploy_set: Some(DeploySet {
                tvc: TestClient::tvc(TEST_DEBOT_TARGET, Some(2)),
                ..Default::default()
            }),
            signer: Signer::Keys { keys: keys.clone() },
            processing_try_index: None,
            address: None,
            call_set: CallSet::some_with_function("constructor"),
        },
        None
    );

    let debot_future = client.deploy_with_giver_async(ParamsOfEncodeMessage {
            abi: debot_abi.clone(),
            deploy_set: Some(DeploySet {
                tvc: TestClient::tvc(TEST_DEBOT, Some(2)),
                ..Default::default()
            }),
            signer: Signer::Keys { keys: keys.clone() },
            processing_try_index: None,
            address: None,
            call_set: Some(CallSet {
                function_name: "constructor".to_owned(),
                header: None,
                input: Some(json!({
                    "targetAbi": hex::encode(&target_abi.json_string().unwrap().as_bytes()),
                    "targetAddr": target_addr,
                }))
            }),
        },
        None
    );

    let (_, debot_addr) = futures::join!(target_future, debot_future);

    let _ = client.net_process_function(
        debot_addr.clone(),
        debot_abi.clone(),
        "setAbi",
        json!({
            "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes())
        }),
        Signer::None,
    ).await.unwrap();

    let data = DebotData {
        debot_addr,
        target_addr,
        keys
    };
    *debot = Some(data.clone());
    data
}

async fn init_debot2(client: Arc<TestClient>) -> DebotData {
    let keys = client.generate_sign_keys();
    let debot_abi = TestClient::abi(TEST_DEBOT2, Some(2));

    let call_set = CallSet::some_with_function_and_input(
        "constructor",
        json!({
            "pub": format!("0x{}", keys.public),
            "sec": format!("0x{}", keys.secret),
        }),
    );
    let deploy_debot_params = ParamsOfEncodeMessage {
        abi: debot_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(TEST_DEBOT2, Some(2))),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set,
    };
    let debot_addr = client.deploy_with_giver_async(deploy_debot_params, Some(1_000_000_000u64)).await;
    let _ = client.net_process_function(
        debot_addr.clone(),
        debot_abi.clone(),
        "setAbi",
        json!({ "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes()) }),
        Signer::Keys { keys: keys.clone() },
    ).await.unwrap();
    let target_addr = String::new();
    DebotData { debot_addr, target_addr, keys }
}

async fn init_debot4(client: Arc<TestClient>) -> DebotData {
    let keys = client.generate_sign_keys();
    let target_abi = TestClient::abi(TEST_DEBOT_TARGET, Some(2));
    let debot_abi = TestClient::abi(TEST_DEBOT4, Some(2));
    let target_deploy_params = ParamsOfEncodeMessage {
        abi: target_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(TEST_DEBOT_TARGET, Some(2))),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };
    let target_addr = client.encode_message(target_deploy_params.clone()).await.unwrap().address;
    client.get_grams_from_giver_async(&target_addr, None).await;
    let call_set = CallSet::some_with_function_and_input(
        "constructor",
        json!({
            "targetAbi": hex::encode(&target_abi.json_string().unwrap().as_bytes()),
            "targetAddr": target_addr,
        }),
    );
    let deploy_debot_params = ParamsOfEncodeMessage {
        abi: debot_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(TEST_DEBOT4, Some(2))),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set,
    };
    let debot_addr = client.deploy_with_giver_async(deploy_debot_params, Some(1_000_000_000u64)).await;
    let _ = client.net_process_function(
        debot_addr.clone(),
        debot_abi.clone(),
        "setAbi",
        json!({
            "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes())
        }),
        Signer::Keys { keys: keys.clone() },
    ).await.unwrap();
    let _ = client.net_process_function(
        debot_addr.clone(),
        debot_abi.clone(),
        "setImage",
        json!({
            "image": TestClient::tvc(TEST_DEBOT_TARGET, Some(2)),
            "pubkey": format!("0x{}", keys.public)
        }),
        Signer::Keys { keys: keys.clone() },
    ).await.unwrap();

    DebotData {
        debot_addr,
        target_addr,
        keys
    }
}

async fn init_debot3(client: Arc<TestClient>) -> DebotData {
    let keys = client.generate_sign_keys();
    let debot_abi = TestClient::abi(TEST_DEBOT3, Some(2));

    let call_set = CallSet::some_with_function("constructor");
    let deploy_debot_params = ParamsOfEncodeMessage {
        abi: debot_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(TEST_DEBOT3, Some(2))),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set,
    };
    let debot_addr = client.deploy_with_giver_async(deploy_debot_params, Some(1_000_000_000u64)).await;
    let _ = client.net_process_function(
        debot_addr.clone(),
        debot_abi.clone(),
        "setABI",
        json!({ "dabi": hex::encode(&debot_abi.json_string().unwrap().as_bytes()) }),
        Signer::Keys { keys: keys.clone() },
    ).await.unwrap();
    DebotData {
        debot_addr,
        target_addr: String::new(),
        keys,
    }
}

const EXIT_CHOICE: u8 = 9;

#[tokio::test(core_threads = 2)]
async fn test_debot_goto() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 1, "inputs": [], "outputs": ["Test Goto Action"] },
        { "choice": 1, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] }
    ]);
    TestBrowser::execute(
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap(),
        vec![],
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_print() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 2, "inputs": [], "outputs": ["Test Print Action", "test2: instant print", "test instant print"] },
        { "choice": 1, "inputs": [], "outputs": ["test simple print"] },
        { "choice": 2, "inputs": [], "outputs": [ format!("integer=1,addr={},string=test_string_1", target_addr)] },
        { "choice": 3, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap(),
        vec![],
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_runact() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 3, "inputs": [], "outputs": ["Test Run Action"] },
        { "choice": 1, "inputs": ["-1:1111111111111111111111111111111111111111111111111111111111111111"], "outputs": ["Test Instant Run", "test1: instant run 1", "test2: instant run 2"] },
        { "choice": 1, "inputs": [], "outputs": ["Test Run Action"] },
        { "choice": 2, "inputs": ["hello"], "outputs": [] },
        { "choice": 3, "inputs": [], "outputs": ["integer=2,addr=-1:1111111111111111111111111111111111111111111111111111111111111111,string=hello"] },
        { "choice": 4, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client,
        debot_addr,
        keys,
        serde_json::from_value(steps).unwrap(),
        vec![]
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_run_method() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 4, "inputs": [], "outputs": ["Test Run Method Action"] },
        { "choice": 1, "inputs": [], "outputs": [] },
        { "choice": 2, "inputs": [], "outputs": ["data=64"] },
        { "choice": 3, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client,
        debot_addr,
        keys,
        serde_json::from_value(steps).unwrap(),
        vec![]
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_send_msg() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 5, "inputs": [], "outputs": ["Test Send Msg Action"] },
        { "choice": 1, "inputs": [], "outputs": ["Sending message {}", "Transaction succeeded."] },
        { "choice": 2, "inputs": [], "outputs": [] },
        { "choice": 3, "inputs": [], "outputs": ["data=100"] },
        { "choice": 4, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client,
        debot_addr,
        keys,
        serde_json::from_value(steps).unwrap(),
        vec![],
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_invoke_debot() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 6, "inputs": [debot_addr.clone()], "outputs": ["Test Invoke Debot Action", "enter debot address:"] },
        { "choice": 1, "inputs": [debot_addr.clone()], "outputs": ["Test Invoke Debot Action", "enter debot address:"],
            "invokes": [
                [
                    { "choice": 1, "inputs": [], "outputs": ["Print test string", "Debot is invoked"] },
                    { "choice": 1, "inputs": [], "outputs": ["Sending message {}", "Transaction succeeded."] }
                ],
            ]
        },
        { "choice": 2, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client,
        debot_addr,
        keys,
        serde_json::from_value(steps).unwrap(),
        vec![]
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_engine_calls() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 7, "inputs": [], "outputs": ["Test Engine Calls"] },
        { "choice": 1, "inputs": [], "outputs": [] },
        { "choice": 2, "inputs": [], "outputs": [] },
        { "choice": 3, "inputs": [], "outputs": [] },
        { "choice": 4, "inputs": [], "outputs": [] },
        { "choice": 5, "inputs": [], "outputs": [] },
        { "choice": 6, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] }
    ]);
    TestBrowser::execute(
        client,
        debot_addr,
        keys,
        serde_json::from_value(steps).unwrap(),
        vec![]
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_interface_call() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 8, "inputs": [], "outputs": ["", "test1 - call interface"] },
        { "choice": 1, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": EXIT_CHOICE, "inputs": [], "outputs": [] }
    ]);
    TestBrowser::execute(
        client,
        debot_addr,
        keys,
        serde_json::from_value(steps).unwrap(),
        vec![],
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_sdk_interface() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot3(client.clone()).await;

    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute(
        client.clone(),
        debot_addr,
        keys,
        steps,
        vec![
            format!("test substring1 passed"),
            format!("test substring2 passed"),
            format!("test mnemonicDeriveSignKeys passed"),
            format!("test genRandom passed"),
            format!("test mnemonic passed"),
            format!("test account passed"),
            format!("test hdkeyXprv passed"),
        ],
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_4() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr, keys } = init_debot4(client.clone()).await;
    let target_abi = TestClient::abi(TEST_DEBOT_TARGET, Some(2));

    let target_boc = download_account(&client, &target_addr).await.expect("account must exist");
    let account: ResultOfParse = client.request_async(
        "boc.parse_account",
        ParamsOfParse {
            boc: target_boc
        },
    ).await.unwrap();
    assert_eq!(account.parsed["acc_type"].as_i64().unwrap(), 0);

    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute(
        client.clone(),
        debot_addr,
        keys,
        steps,
        vec![
            format!("Target contract deployed."),
            format!("Enter 1"),
            format!("getData"),
            format!("setData(128)"),
            format!("Sign external message:"),
            format!("Transaction succeeded"),
            format!("setData2(129)"),
        ],
    ).await;

    let target_boc = download_account(&client, &target_addr).await.expect("account must exist");
    let account: ResultOfParse = client.request_async(
        "boc.parse_account",
        ParamsOfParse {
            boc: target_boc
        },
    ).await.unwrap();
    assert_eq!(account.parsed["acc_type"].as_i64().unwrap(), 1);

    assert_get_method(
        &client,
        &target_addr,
        &target_abi,
        "getData",
        json!({"key": 1}),
        json!({"num": format!("0x{:064x}", 129) })
    ).await;

}

#[tokio::test(core_threads = 2)]
async fn test_debot_msg_interface() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot2(client.clone()).await;
    let debot_abi = TestClient::abi(TEST_DEBOT2, Some(2));
    let counter = 10;
    let counter_after = 15;

    assert_get_method(
        &client,
        &debot_addr,
        &debot_abi,
        "counter",
        json!({}),
        json!({"counter": format!("{}", counter) })
    ).await;

    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![
            format!("counter={}", counter),
            format!("Increment succeeded"),
            format!("counter={}", counter_after),
        ],
    ).await;

    assert_get_method(
        &client,
        &debot_addr,
        &debot_abi,
        "counter",
        json!({}),
        json!({"counter": format!("{}", counter_after) })
    ).await;
}

async fn download_account(client: &Arc<TestClient>, addr: &str) -> Option<String> {
    let client = client.clone();
    let accounts: ResultOfQueryCollection = client.request_async(
        "net.query_collection",
        ParamsOfQueryCollection {
            collection: format!("accounts"),
            filter: Some(json!({
                "id": { "eq": addr }
            })),
            result: format!("boc"),
            limit: Some(1),
            order: None,
        }
    ).await.unwrap();

    if accounts.result.len() == 1 {
        Some(accounts.result[0]["boc"].as_str().unwrap().to_owned())
    } else {
        None
    }
}
async fn assert_get_method(client: &Arc<TestClient>, addr: &String, abi: &Abi, func: &str, params: Value, returns: Value) {
    let client = client.clone();
    let acc_boc = download_account(&client, &addr).await.expect("Account not found");

    let call_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: None,
        signer: Signer::None,
        processing_try_index: None,
        address: Some(addr.clone()),
        call_set: CallSet::some_with_function_and_input(func, params),
    };

    let message = client.encode_message(call_params).await.unwrap().message;

    let result: ResultOfRunTvm = client.request_async(
        "tvm.run_tvm",
        ParamsOfRunTvm {
            account: acc_boc,
            message,
            abi: Some(abi.clone()),
            execution_options: None,
            boc_cache: None,
            return_updated_account: Some(true),
        },
    ).await.unwrap();

    let output = result.decoded.unwrap().output.expect("output must exist");
    assert_eq!(returns, output);
}