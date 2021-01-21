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

use crate::client::ParamsOfAppRequest;
use crate::json_interface::interop::ResponseType;
use crate::tests::{TEST_DEBOT, TEST_DEBOT_TARGET, TestClient};
use crate::crypto::KeyPair;
use crate::json_interface::debot::*;
use crate::abi::{CallSet, DeploySet, ParamsOfEncodeMessage, Signer, Abi, ParamsOfDecodeMessageBody, DecodedMessageBody};
use crate::boc::{ParamsOfParse, ResultOfParse};
use futures::future::{BoxFuture, FutureExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::VecDeque;
use tokio::sync::Mutex;
use ton_abi::Contract;
use super::*;

lazy_static!(
    static ref DEBOT: Mutex<Option<DebotData>> = Mutex::new(None);
);

const SUPPORTED_INTERFACES: &[&str] = &["f6927c0d4bdb69e1b52d27f018d156ff04152f00558042ff674f0fec32e4369d"];
const INTERFACE_ABI: &str = r#"
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
	]
}
"#;

struct Echo {}
impl Echo {
    fn echo(answer_id: u32, request: &str) -> (String, JsonValue) {
        let abi = Contract::load(INTERFACE_ABI.to_string().as_bytes()).unwrap();
        let func_name = abi.function_by_id(answer_id, true).unwrap().name.clone();
        (func_name, json!({ "response": request.as_bytes() }))
    }

    fn call(func: &str, args: &JsonValue) -> (String, JsonValue) {
        match func {
            "echo" => {
                let answer_id = u32::from_str_radix(args["answerId"].as_str().unwrap(), 10).unwrap();
                let request_vec = hex::decode(args["request"].as_str().unwrap()).unwrap();
                let request = std::str::from_utf8(&request_vec).unwrap();
                Self::echo(answer_id, request)
            },
            _ => (String::new(), json!({})),
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
}

impl TestBrowser {

    pub async fn execute_from_state(client: Arc<TestClient>, state: Arc<BrowserData>, start_function: &str) {
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
            ParamsOfStart {
                address: state.address.clone()
            },
            callback
        ).await.unwrap();

        while !state.finished.load(Ordering::Relaxed) {
            Self::execute_interface_calls(&handle, client.clone(), state.clone()).await;

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

        let _: () = client.request_async(
            "debot.remove",
            handle).await.unwrap();
    }

    pub async fn execute(client: Arc<TestClient>, address: String, keys: KeyPair, steps: Vec<DebotStep>) {
        let state = Arc::new(BrowserData {
            current: Mutex::new(Default::default()), 
            next: Mutex::new(steps),
            client: client.clone(),
            keys,
            address: address.clone(),
            finished: AtomicBool::new(false),
            switch_started: AtomicBool::new(false),
            msg_queue: Mutex::new(Default::default()),
        });

        Self::execute_from_state(client, state, "debot.start").await
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
                });
                Self::call_execute_boxed(client, state, "debot.fetch").await;
                ResultOfAppDebotBrowser::InvokeDebot
            },
            _ => panic!("invalid call {:#?}", params)
        }
    }

    async fn execute_interface_calls(
        handle: &RegisteredDebot,
        client: Arc<TestClient>,
        state: Arc<BrowserData>
    ) {
        let mut msg_queue = state.msg_queue.lock().await;
        for msg in msg_queue.drain(0..) {
            let parsed: ResultOfParse = client.request_async(
                "boc.parse_message",
                ParamsOfParse { boc: msg.clone() },
            ).await.unwrap();
            let body = parsed.parsed["body"].as_str().unwrap().to_owned();
            let wc_and_addr: Vec<_> = parsed.parsed["dst"]
                .as_str()
                .unwrap()
                .split(':')
                .collect();
            let interface_id = wc_and_addr[1];
            let wc = i8::from_str_radix(wc_and_addr[0], 10).unwrap();
            assert_eq!(wc, DEBOT_WC);
            assert_eq!(SUPPORTED_INTERFACES.contains(&interface_id), true);
            let decoded: DecodedMessageBody = client.request_async(
                "abi.decode_message_body",
                ParamsOfDecodeMessageBody {
                    abi: Abi::Json(INTERFACE_ABI.to_owned()),
                    body,
                    is_internal: true,
                },
            ).await.unwrap();
            println!("call for interface id {}", interface_id);
            println!("function {} ({})", decoded.name, decoded.value.as_ref().unwrap());
            let (func_name, return_args) = Echo::call(&decoded.name, &decoded.value.unwrap());
            let params = ParamsOfEncodeMessage {
                abi: Abi::Json(INTERFACE_ABI.to_owned()),
                deploy_set: None,
                signer: Signer::None,
                processing_try_index: None,
                address: None,
                call_set: CallSet::some_with_function_and_input(&func_name, return_args),
            };
            let response_msg = client.encode_message(params).await.unwrap().message;
            let _result: () = client.request_async(
                "debot.send",
                ParamsOfSend {
                    debot_handle: handle.debot_handle.clone(),
                    message: response_msg,
                }
            ).await.unwrap();
        }
    }
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
            initial_data: None,
            tvc: TestClient::tvc(TEST_DEBOT_TARGET, Some(2)),
            workchain_id: None,
        }),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };

    let target_addr = client.encode_message(target_deploy_params.clone()).await.unwrap().address;

    let target_future = client.deploy_with_giver_async(ParamsOfEncodeMessage {
            abi: target_abi.clone(),
            deploy_set: Some(DeploySet {
                initial_data: None,
                tvc: TestClient::tvc(TEST_DEBOT_TARGET, Some(2)),
                workchain_id: None,
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
                initial_data: None,
                tvc: TestClient::tvc(TEST_DEBOT, Some(2)),
                workchain_id: None,
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
        serde_json::from_value(steps).unwrap()
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
        serde_json::from_value(steps).unwrap()
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
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
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
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
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
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
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
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
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
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
    ).await;
}