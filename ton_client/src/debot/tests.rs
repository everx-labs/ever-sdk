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

use super::tests_interfaces::*;
use super::*;
use crate::abi::{
    Abi, CallSet, DecodedMessageBody, DeploySet, ParamsOfDecodeMessageBody,
    ParamsOfEncodeInternalMessage, ParamsOfEncodeMessage, ResultOfEncodeInternalMessage, Signer,
};
use crate::boc::{
    ParamsOfGetBocHash, ParamsOfGetCodeFromTvc, ParamsOfParse, ResultOfGetBocHash,
    ResultOfGetCodeFromTvc, ResultOfParse,
};
use crate::client::ParamsOfAppRequest;
use crate::crypto::KeyPair;
use crate::json_interface::debot::*;
use crate::json_interface::interop::ResponseType;
use crate::net::ParamsOfQueryCollection;
use crate::net::ResultOfQueryCollection;
use crate::tests::{TestClient, TEST_DEBOT, TEST_DEBOT_TARGET};
use crate::tvm::{ParamsOfRunTvm, ResultOfRunTvm};
use futures::future::{BoxFuture, FutureExt};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    static ref DEBOT: Mutex<Option<DebotData>> = Mutex::new(None);
}

const TEST_DEBOT2: &'static str = "testDebot2";
const TEST_DEBOT3: &'static str = "testDebot3";
const TEST_DEBOT4: &'static str = "testDebot4";
const TEST_DEBOT5: &'static str = "testDebot5";
const TEST_DEBOTA: &'static str = "tda";
const TEST_DEBOTB: &'static str = "tdb";

struct ExpectedTransaction {
    dst: String,
    out: Vec<Spending>,
    setcode: bool,
    signkey: String,
    approved: bool,
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
    pub finished: AtomicBool,
    pub switch_started: AtomicBool,
    pub msg_queue: Mutex<VecDeque<String>>,
    pub terminal: Mutex<Terminal>,
    pub echo: Echo,
    pub sign_box_input: SingingBoxInput,
    pub encrypt_box_input: EncryptionBoxInput,
    pub bots: Mutex<HashMap<String, RegisteredDebot>>,
    pub info: DebotInfo,
    pub activity: Mutex<Vec<ExpectedTransaction>>,
}

impl TestBrowser {
    async fn fetch_debot(
        client: Arc<TestClient>,
        state: Arc<BrowserData>,
        address: String,
    ) -> RegisteredDebot {
        let state_copy = state.clone();
        let client_copy = client.clone();
        let callback = move |params, response_type| {
            log::debug!("received from debot: {:#}", params);
            let client = client_copy.clone();
            let state = state_copy.clone();
            async move {
                match response_type {
                    ResponseType::AppNotify => {
                        Self::process_notification(&state, serde_json::from_value(params).unwrap())
                            .await;
                    }
                    ResponseType::AppRequest => {
                        tokio::spawn(async move {
                            let request: ParamsOfAppRequest =
                                serde_json::from_value(params).unwrap();
                            let result = Self::process_call(
                                client.clone(),
                                &state,
                                serde_json::from_value(request.request_data).unwrap(),
                            )
                            .await;
                            client
                                .resolve_app_request(request.app_request_id, result)
                                .await;
                        });
                    }
                    _ => panic!("Wrong response type"),
                }
            }
        };

        let handle: RegisteredDebot = client
            .request_async_callback(
                "debot.init",
                ParamsOfInit {
                    address: address.clone(),
                },
                callback,
            )
            .await
            .unwrap();

        let handle_copy = RegisteredDebot {
            debot_handle: handle.debot_handle.clone(),
            debot_abi: handle.debot_abi.clone(),
            info: handle.info.clone(),
        };
        state.bots.lock().await.insert(address.clone(), handle_copy);
        handle
    }

    pub async fn execute_from_state(
        client: Arc<TestClient>,
        state: Arc<BrowserData>,
        call_start: bool,
    ) {
        if call_start {
            let res: ResultOfFetch = client
                .request_async(
                    "debot.fetch",
                    ParamsOfFetch {
                        address: state.address.clone(),
                    },
                )
                .await
                .unwrap();
            assert_eq!(res.info, state.info);
        }
        let handle = Self::fetch_debot(client.clone(), state.clone(), state.address.clone()).await;

        if call_start {
            let _: () = client
                .request_async(
                    "debot.start",
                    ParamsOfStart {
                        debot_handle: handle.debot_handle.clone(),
                    },
                )
                .await
                .unwrap();
        }

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
            let _: () = client
                .request_async(
                    "debot.execute",
                    ParamsOfExecute {
                        debot_handle: handle.debot_handle.clone(),
                        action,
                    },
                )
                .await
                .unwrap();

            let step = state.current.lock().await;
            assert_eq!(step.outputs.len(), step.step.outputs.len());
            step.outputs
                .iter()
                .zip(step.step.outputs.iter())
                .for_each(|outs| {
                    match outs.1.find("{}") {
                        Some(pos) => {
                            assert_eq!(outs.0.get(..pos).unwrap(), outs.1.get(..pos).unwrap(),)
                        }
                        None => assert_eq!(outs.0, outs.1),
                    };
                });
            assert_eq!(step.step.inputs.len(), 0);
            assert_eq!(step.step.invokes.len(), 0);

            if step.available_actions.len() == 0 {
                break;
            }
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
        abi: String,
    ) {
        let mut info = DebotInfo::default();
        info.dabi = Some(abi);
        let state = Arc::new(BrowserData {
            current: Mutex::new(Default::default()),
            next: Mutex::new(steps),
            keys: keys.clone(),
            address: address.clone(),
            finished: AtomicBool::new(false),
            switch_started: AtomicBool::new(false),
            msg_queue: Mutex::new(Default::default()),
            terminal: Mutex::new(Terminal::new(terminal_outputs)),
            echo: Echo::new(),
            sign_box_input: SingingBoxInput::new(client.clone(), keys.clone()).await,
            encrypt_box_input: EncryptionBoxInput::new(client.clone()).await,
            bots: Mutex::new(HashMap::new()),
            info,
            activity: Mutex::new(vec![]),
        });

        Self::execute_from_state(client, state, true).await
    }

    pub async fn execute_with_details(
        client: Arc<TestClient>,
        address: String,
        keys: KeyPair,
        steps: Vec<DebotStep>,
        terminal_outputs: Vec<String>,
        info: DebotInfo,
        activity: Vec<ExpectedTransaction>,
    ) {
        let state = Arc::new(BrowserData {
            current: Mutex::new(Default::default()),
            next: Mutex::new(steps),
            keys: keys.clone(),
            address: address.clone(),
            finished: AtomicBool::new(false),
            switch_started: AtomicBool::new(false),
            msg_queue: Mutex::new(Default::default()),
            terminal: Mutex::new(Terminal::new(terminal_outputs)),
            echo: Echo::new(),
            sign_box_input: SingingBoxInput::new(client.clone(), keys.clone()).await,
            encrypt_box_input: EncryptionBoxInput::new(client.clone()).await,
            bots: Mutex::new(HashMap::new()),
            info,
            activity: Mutex::new(activity),
        });

        Self::execute_from_state(client, state, true).await
    }

    async fn process_notification(state: &BrowserData, params: ParamsOfAppDebotBrowser) {
        match params {
            ParamsOfAppDebotBrowser::Log { msg } => {
                state.current.lock().await.outputs.push(msg);
            }
            ParamsOfAppDebotBrowser::Switch { context_id } => {
                assert_eq!(state.switch_started.swap(true, Ordering::Relaxed), false);
                if context_id == STATE_EXIT {
                    state.finished.store(true, Ordering::Relaxed);
                }
                state.current.lock().await.available_actions.clear();
            }
            ParamsOfAppDebotBrowser::SwitchCompleted => {
                assert_eq!(state.switch_started.swap(false, Ordering::Relaxed), true);
            }
            ParamsOfAppDebotBrowser::ShowAction { action } => {
                state.current.lock().await.available_actions.push(action);
            }
            ParamsOfAppDebotBrowser::Send { message } => {
                state.msg_queue.lock().await.push_back(message);
            }
            _ => panic!("invalid notification {:#?}", params),
        }
    }

    fn call_execute_boxed(
        client: Arc<TestClient>,
        state: Arc<BrowserData>,
        call_start: bool,
    ) -> BoxFuture<'static, ()> {
        Self::execute_from_state(client, state, call_start).boxed()
    }

    async fn process_call(
        client: Arc<TestClient>,
        state: &BrowserData,
        params: ParamsOfAppDebotBrowser,
    ) -> ResultOfAppDebotBrowser {
        match params {
            ParamsOfAppDebotBrowser::Input { prompt: _ } => {
                let value = state.current.lock().await.step.inputs.remove(0);
                ResultOfAppDebotBrowser::Input {
                    value: value.to_owned(),
                }
            }
            ParamsOfAppDebotBrowser::GetSigningBox => {
                let signing_box: crate::crypto::RegisteredSigningBox = client
                    .request_async("crypto.get_signing_box", state.keys.clone())
                    .await
                    .unwrap();

                ResultOfAppDebotBrowser::GetSigningBox {
                    signing_box: signing_box.handle,
                }
            }
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
                    keys: state.keys.clone(),
                    address: debot_addr,
                    finished: AtomicBool::new(false),
                    switch_started: AtomicBool::new(false),
                    msg_queue: Mutex::new(Default::default()),
                    terminal: Mutex::new(Terminal::new(vec![])),
                    echo: Echo::new(),
                    sign_box_input: SingingBoxInput::new(client.clone(), state.keys.clone()).await,
                    encrypt_box_input: EncryptionBoxInput::new(client.clone()).await,
                    bots: Mutex::new(HashMap::new()),
                    info: Default::default(),
                    activity: Mutex::new(vec![]),
                });
                Self::call_execute_boxed(client, state, false).await;
                ResultOfAppDebotBrowser::InvokeDebot
            }
            ParamsOfAppDebotBrowser::Approve { activity } => {
                let mut approved = true;
                if let Some(expected) = state.activity.lock().await.pop() {
                    approved = expected.approved;
                    match activity {
                        DebotActivity::Transaction {
                            msg: _,
                            dst,
                            out,
                            fee,
                            setcode,
                            signkey,
                            signing_box_handle,
                        } => {
                            assert_eq!(expected.dst, dst);
                            assert_eq!(expected.out, out);
                            assert_eq!(expected.setcode, setcode);
                            assert_eq!(expected.signkey, signkey);
                            assert!(signing_box_handle != 0);
                            assert!(fee > 0);
                        }
                    }
                }
                ResultOfAppDebotBrowser::Approve { approved }
            }
            _ => panic!("invalid call {:#?}", params),
        }
    }

    async fn handle_message_queue(client: Arc<TestClient>, state: Arc<BrowserData>) {
        let mut msg_opt = state.msg_queue.lock().await.pop_front();
        while let Some(msg) = msg_opt {
            let parsed: ResultOfParse = client
                .request_async("boc.parse_message", ParamsOfParse { boc: msg.clone() })
                .await
                .unwrap();

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
                } else if SUPPORTED_INTERFACES[2] == interface_id {
                    Abi::Json(SIGNING_BOX_ABI.to_owned())
                } else if SUPPORTED_INTERFACES[3] == interface_id {
                    Abi::Json(ENCRYPTION_BOX_ABI.to_owned())
                } else {
                    panic!("unsupported interface");
                };
                let decoded: DecodedMessageBody = client
                    .request_async(
                        "abi.decode_message_body",
                        ParamsOfDecodeMessageBody {
                            abi,
                            body,
                            is_internal: true,
                        },
                    )
                    .await
                    .unwrap();
                let (func, args) = (decoded.name, decoded.value.unwrap());
                log::info!("request: {} ({})", func, args);
                let (func_id, return_args) = if SUPPORTED_INTERFACES[0] == interface_id {
                    state.echo.call(&func, &args)
                } else if SUPPORTED_INTERFACES[1] == interface_id {
                    state.terminal.lock().await.call(&func, &args)
                } else if SUPPORTED_INTERFACES[2] == interface_id {
                    state.sign_box_input.call(&func, &args)
                } else {
                    state.encrypt_box_input.call(&func, &args).await
                };
                log::info!("response: {} ({})", func_id, return_args);

                let call_set = match func_id {
                    0 => None,
                    _ => CallSet::some_with_function_and_input(
                        &format!("0x{:x}", func_id),
                        return_args,
                    ),
                };
                let bots = state.bots.lock().await;
                let handle = bots.get(src_addr).unwrap();
                let message = encode_internal_message(
                    client.clone(),
                    &handle.debot_abi,
                    src_addr.to_owned(),
                    call_set,
                )
                .await;
                debot_send(client.clone(), handle.debot_handle.clone(), message).await;
            } else {
                let debot_fetched = state.bots.lock().await.get(dest_addr).is_some();
                if !debot_fetched {
                    TestBrowser::fetch_debot(client.clone(), state.clone(), dest_addr.to_owned())
                        .await;
                }
                let debot_handle = state
                    .bots
                    .lock()
                    .await
                    .get(dest_addr)
                    .unwrap()
                    .debot_handle
                    .clone();
                debot_send(client.clone(), debot_handle, msg).await;
            }

            msg_opt = state.msg_queue.lock().await.pop_front();
        }
    }
}

async fn encode_internal_message(
    client: Arc<TestClient>,
    abi: &str,
    addr: String,
    call_set: Option<CallSet>,
) -> String {
    let r: ResultOfEncodeInternalMessage = client
        .request_async(
            "abi.encode_internal_message",
            ParamsOfEncodeInternalMessage {
                abi: Some(Abi::Contract(serde_json::from_str(abi).unwrap())),
                address: Some(addr),
                src_address: None,
                deploy_set: None,
                call_set,
                value: "1000000000000000".to_owned(),
                bounce: None,
                enable_ihr: None,
            },
        )
        .await
        .unwrap();
    r.message
}

async fn debot_send(client: Arc<TestClient>, debot_handle: DebotHandle, message: String) {
    let _result: () = client
        .request_async(
            "debot.send",
            ParamsOfSend {
                debot_handle,
                message,
            },
        )
        .await
        .unwrap();
}

#[derive(Clone)]
struct DebotData {
    debot_addr: String,
    target_addr: String,
    keys: KeyPair,
    abi: String,
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

    let target_addr = client
        .encode_message(target_deploy_params.clone())
        .await
        .unwrap()
        .address;

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
        None,
    );

    let debot_future = client.deploy_with_giver_async(
        ParamsOfEncodeMessage {
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
                })),
            }),
        },
        None,
    );

    let (_, debot_addr) = futures::join!(target_future, debot_future);

    let _ = client
        .net_process_function(
            debot_addr.clone(),
            debot_abi.clone(),
            "setAbi",
            json!({
                "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes())
            }),
            Signer::None,
        )
        .await
        .unwrap();

    let data = DebotData {
        debot_addr,
        target_addr,
        keys,
        abi: debot_abi.json_string().unwrap(),
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
    let debot_addr = client
        .deploy_with_giver_async(deploy_debot_params, Some(1_000_000_000u64))
        .await;
    let _ = client
        .net_process_function(
            debot_addr.clone(),
            debot_abi.clone(),
            "setAbi",
            json!({ "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes()) }),
            Signer::Keys { keys: keys.clone() },
        )
        .await
        .unwrap();
    let target_addr = String::new();
    DebotData {
        debot_addr,
        target_addr,
        keys,
        abi: debot_abi.json_string().unwrap(),
    }
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
    let target_addr = client
        .encode_message(target_deploy_params.clone())
        .await
        .unwrap()
        .address;
    client.get_tokens_from_giver_async(&target_addr, None).await;
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
    let debot_addr = client
        .deploy_with_giver_async(deploy_debot_params, Some(1_000_000_000u64))
        .await;
    let _ = client
        .net_process_function(
            debot_addr.clone(),
            debot_abi.clone(),
            "setAbi",
            json!({
                "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes())
            }),
            Signer::Keys { keys: keys.clone() },
        )
        .await
        .unwrap();
    let _ = client
        .net_process_function(
            debot_addr.clone(),
            debot_abi.clone(),
            "setImage",
            json!({
                "image": TestClient::tvc(TEST_DEBOT_TARGET, Some(2)),
                "pubkey": format!("0x{}", keys.public)
            }),
            Signer::Keys { keys: keys.clone() },
        )
        .await
        .unwrap();

    DebotData {
        debot_addr,
        target_addr,
        keys,
        abi: debot_abi.json_string().unwrap(),
    }
}

async fn init_debot3(client: Arc<TestClient>) -> DebotData {
    init_simple_debot(client, TEST_DEBOT3).await
}

async fn init_simple_debot(client: Arc<TestClient>, name: &str) -> DebotData {
    let keys = client.generate_sign_keys();
    let debot_abi = TestClient::abi(name, Some(2));

    let call_set = CallSet::some_with_function("constructor");
    let deploy_debot_params = ParamsOfEncodeMessage {
        abi: debot_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(name, Some(2))),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set,
    };
    let debot_addr = client
        .deploy_with_giver_async(deploy_debot_params, Some(100_000_000_000u64))
        .await;
    let _ = client
        .net_process_function(
            debot_addr.clone(),
            debot_abi.clone(),
            "setABI",
            json!({ "dabi": hex::encode(&debot_abi.json_string().unwrap().as_bytes()) }),
            Signer::Keys { keys: keys.clone() },
        )
        .await
        .unwrap();
    DebotData {
        debot_addr,
        target_addr: String::new(),
        keys,
        abi: debot_abi.json_string().unwrap(),
    }
}

async fn init_debot5(client: Arc<TestClient>, count: u32) -> (String, String) {
    let debot_abi = TestClient::abi(TEST_DEBOT5, Some(2));
    let hash_str = get_code_hash_from_tvc(client.clone(), TEST_DEBOT5).await;
    let call_set = CallSet::some_with_function_and_input(
        "constructor",
        json!({ "codeHash": format!("0x{}", hash_str) }),
    );
    let mut deploy_debot_params = ParamsOfEncodeMessage {
        abi: debot_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(TEST_DEBOT5, Some(2))),
        call_set,
        ..Default::default()
    };

    let mut addrs = vec![];
    for i in 0..count {
        let keys = client.generate_sign_keys();
        deploy_debot_params.signer = Signer::Keys { keys: keys.clone() };
        let debot_addr = client
            .deploy_with_giver_async(deploy_debot_params.clone(), Some(1_000_000_000u64))
            .await;
        addrs.push(debot_addr.clone());
        if i == 0 {
            let _ = client
                .net_process_function(
                    debot_addr.clone(),
                    debot_abi.clone(),
                    "setABI",
                    json!({ "dabi": hex::encode(&debot_abi.json_string().unwrap().as_bytes()) }),
                    Signer::Keys { keys: keys.clone() },
                )
                .await
                .unwrap();
        }
    }
    (addrs[0].clone(), debot_abi.json_string().unwrap())
}

async fn init_debot_pair(
    client: Arc<TestClient>,
    debot1: &str,
    debot2: &str,
) -> (String, String, String) {
    let keys = client.generate_sign_keys();
    let debot1_abi = TestClient::abi(debot1, Some(2));
    let debot2_abi = TestClient::abi(debot2, Some(2));

    let deploy_params2 = ParamsOfEncodeMessage {
        abi: debot2_abi.clone(),
        deploy_set: Some(DeploySet {
            tvc: TestClient::tvc(debot2, Some(2)),
            ..Default::default()
        }),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set: CallSet::some_with_function("constructor"),
    };
    let debot2_addr = client
        .encode_message(deploy_params2.clone())
        .await
        .unwrap()
        .address;

    let call_set =
        CallSet::some_with_function_and_input("constructor", json!({ "targetAddr": debot2_addr }));
    let deploy_params1 = ParamsOfEncodeMessage {
        abi: debot1_abi.clone(),
        deploy_set: DeploySet::some_with_tvc(TestClient::tvc(debot1, Some(2))),
        signer: Signer::Keys { keys: keys.clone() },
        processing_try_index: None,
        address: None,
        call_set,
    };
    let debot1_addr = client
        .deploy_with_giver_async(deploy_params1, Some(1_000_000_000u64))
        .await;
    let _ = client
        .deploy_with_giver_async(deploy_params2, Some(1_000_000_000u64))
        .await;

    let future1 = client.net_process_function(
        debot1_addr.clone(),
        debot1_abi.clone(),
        "setAbi",
        json!({ "debotAbi": hex::encode(&debot1_abi.json_string().unwrap().as_bytes()) }),
        Signer::Keys { keys: keys.clone() },
    );

    let future2 = client.net_process_function(
        debot2_addr.clone(),
        debot2_abi.clone(),
        "setAbi",
        json!({ "debotAbi": hex::encode(&debot2_abi.json_string().unwrap().as_bytes()) }),
        Signer::Keys { keys: keys.clone() },
    );

    let (_, _) = futures::join!(future1, future2);

    (debot1_addr, debot2_addr, debot1_abi.json_string().unwrap())
}

async fn init_hello_debot(client: Arc<TestClient>) -> DebotData {
    let data = init_simple_debot(client.clone(), "helloDebot").await;
    let abi = Abi::Contract(serde_json::from_str(&data.abi).unwrap());
    let _ = client
        .net_process_function(
            data.debot_addr.clone(),
            abi,
            "setIcon",
            json!({ "icon": hex::encode(TestClient::icon("helloDebot", Some(2))) }),
            Signer::Keys {
                keys: data.keys.clone(),
            },
        )
        .await
        .unwrap();
    data
}

async fn count_accounts_by_codehash(client: Arc<TestClient>, code_hash: String) -> u32 {
    let res: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: "accounts".to_owned(),
                filter: Some(json!({
                    "code_hash": { "eq": code_hash}
                })),
                result: "id".to_owned(),
                limit: None,
                order: None,
            },
        )
        .await
        .unwrap();

    res.result.len() as u32
}

async fn get_code_hash_from_tvc(client: Arc<TestClient>, name: &str) -> String {
    let debot_tvc = TestClient::tvc(name, Some(2));
    let result: ResultOfGetCodeFromTvc = client
        .request_async(
            "boc.get_code_from_tvc",
            ParamsOfGetCodeFromTvc { tvc: debot_tvc },
        )
        .await
        .unwrap();

    let result: ResultOfGetBocHash = client
        .request_async("boc.get_boc_hash", ParamsOfGetBocHash { boc: result.code })
        .await
        .unwrap();

    result.hash
}

const EXIT_CHOICE: u8 = 9;

#[tokio::test(core_threads = 2)]
async fn test_debot_goto() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_print() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_runact() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        vec![],
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_run_method() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        vec![],
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_send_msg() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_invoke_debot() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        vec![],
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_engine_calls() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        vec![],
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_interface_call() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot(client.clone()).await;

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
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_inner_interfaces() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot3(client.clone()).await;

    let steps = serde_json::from_value(json!([])).unwrap();

    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![
            format!("test substring1 passed"),
            format!("test substring2 passed"),
            format!("test mnemonicDeriveSignKeys passed"),
            format!("test genRandom passed"),
            format!("test naclbox passed"),
            format!("test naclKeypairFromSecret passed"),
            format!("test hex encode passed"),
            format!("test base64 encode passed"),
            format!("test mnemonic passed"),
            format!("test naclboxopen passed"),
            format!("test account passed"),
            format!("test hdkeyXprv passed"),
            format!("test sign hash passed"),
            format!("test hex decode passed"),
            format!("test base64 decode passed"),
        ],
        DebotInfo {
            name: Some("TestSdk".to_owned()),
            version: Some("0.4.0".to_owned()),
            publisher: Some("TON Labs".to_owned()),
            caption: Some("Test for SDK interface".to_owned()),
            author: Some("TON Labs".to_owned()),
            support: Some(
                "0:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            ),
            hello: Some("Hello, I'm a test.".to_owned()),
            language: Some("en".to_owned()),
            dabi: Some(abi),
            icon: Some(format!("")),
            interfaces: vec![
                "0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3".to_owned(),
            ],
        },
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_4() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr,
        keys,
        abi,
    } = init_debot4(client.clone()).await;
    let target_abi = TestClient::abi(TEST_DEBOT_TARGET, Some(2));

    let target_boc = download_account(&client, &target_addr)
        .await
        .expect("account must exist");
    let account: ResultOfParse = client
        .request_async("boc.parse_account", ParamsOfParse { boc: target_boc })
        .await
        .unwrap();
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
        abi,
    )
    .await;

    let target_boc = download_account(&client, &target_addr)
        .await
        .expect("account must exist");
    let account: ResultOfParse = client
        .request_async("boc.parse_account", ParamsOfParse { boc: target_boc })
        .await
        .unwrap();
    assert_eq!(account.parsed["acc_type"].as_i64().unwrap(), 1);

    assert_get_method(
        &client,
        &target_addr,
        &target_abi,
        "getData",
        json!({"key": 1}),
        json!({ "num": format!("0x{:064x}", 129) }),
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_msg_interface() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_debot2(client.clone()).await;
    let debot_abi = TestClient::abi(TEST_DEBOT2, Some(2));
    let counter = 10;
    let counter_after = 15;

    assert_get_method(
        &client,
        &debot_addr,
        &debot_abi,
        "counter",
        json!({}),
        json!({ "counter": format!("{}", counter) }),
    )
    .await;

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
        abi,
    )
    .await;

    assert_get_method(
        &client,
        &debot_addr,
        &debot_abi,
        "counter",
        json!({}),
        json!({ "counter": format!("{}", counter_after) }),
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_invoke_msgs() {
    let client = std::sync::Arc::new(TestClient::new());
    let (debot1, _, abi) = init_debot_pair(client.clone(), TEST_DEBOTA, TEST_DEBOTB).await;
    let keys = client.generate_sign_keys();

    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute(
        client.clone(),
        debot1.clone(),
        keys,
        steps,
        vec![
            format!("Invoking Debot B"),
            format!("DebotB receives question: What is your name?"),
            format!("DebotA receives answer: My name is DebotB"),
        ],
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_sdk_get_accounts_by_hash() {
    let client = std::sync::Arc::new(TestClient::new());
    let deploy_count = 8;
    let (debot, abi) = init_debot5(client.clone(), deploy_count).await;
    let code_hash = get_code_hash_from_tvc(client.clone(), TEST_DEBOT5).await;
    let total_count = count_accounts_by_codehash(client.clone(), code_hash).await;
    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute(
        client.clone(),
        debot.clone(),
        KeyPair::default(),
        steps,
        vec![format!("{} contracts.", total_count)],
        abi,
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_getinfo() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_hello_debot(client.clone()).await;
    let icon = TestClient::icon("helloDebot", Some(2));
    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![
            format!("Hello, World!"),
            format!("How is it going?"),
            format!("You have entered \"testinput\""),
        ],
        DebotInfo {
            name: Some("HelloWorld".to_owned()),
            version: Some("0.2.0".to_owned()),
            publisher: Some("TON Labs".to_owned()),
            caption: Some("Start develop DeBot from here".to_owned()),
            author: Some("TON Labs".to_owned()),
            support: Some(
                "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94".to_owned(),
            ),
            hello: Some("Hello, i am a HelloWorld DeBot.".to_owned()),
            language: Some("en".to_owned()),
            dabi: Some(abi),
            icon: Some(icon),
            interfaces: vec![
                "0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3".to_owned(),
            ],
        },
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_approve() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot6").await;
    let info = DebotInfo {
        name: Some("testDebot6".to_owned()),
        version: Some("0.1.0".to_owned()),
        publisher: Some("TON Labs".to_owned()),
        caption: Some("Test for approve callback and signing handle".to_owned()),
        author: Some("TON Labs".to_owned()),
        support: Some(
            "0:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        ),
        hello: Some("testDebot6".to_owned()),
        language: Some("en".to_owned()),
        dabi: Some(abi),
        icon: Some(format!("")),
        interfaces: vec![
            "0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3".to_owned(),
            "0xc13024e101c95e71afb1f5fa6d72f633d51e721de0320d73dfd6121a54e4d40a".to_owned(),
        ],
    };
    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        steps,
        vec![format!("Send1 succeeded"), format!("Send2 rejected")],
        info,
        vec![
            ExpectedTransaction {
                dst: debot_addr.clone(),
                out: vec![],
                setcode: false,
                signkey: keys.public.clone(),
                approved: true,
            },
            ExpectedTransaction {
                dst: debot_addr.clone(),
                out: vec![Spending {
                    amount: 10000000000,
                    dst: debot_addr.clone(),
                }],
                setcode: false,
                signkey: keys.public.clone(),
                approved: false,
            },
            ExpectedTransaction {
                dst: debot_addr.clone(),
                out: vec![
                    Spending {
                        amount: 2200000000,
                        dst: debot_addr.clone(),
                    },
                    Spending {
                        amount: 3500000000,
                        dst: format!("0:{:064}", 0),
                    },
                ],
                setcode: false,
                signkey: keys.public.clone(),
                approved: true,
            },
        ],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_json_interface() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot7").await;
    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![],
        DebotInfo {
            name: Some("Test DeBot 7".to_owned()),
            version: Some("0.1.0".to_owned()),
            publisher: Some("TON Labs".to_owned()),
            caption: Some("Test for Json interface".to_owned()),
            author: Some("TON Labs".to_owned()),
            support: Some(
                "0:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            ),
            hello: Some("Test DeBot 7".to_owned()),
            language: Some("en".to_owned()),
            dabi: Some(abi),
            icon: Some(format!("")),
            interfaces: vec![
                "0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3".to_owned(),
                "0x442288826041d564ccedc579674f17c1b0a3452df799656a9167a41ab270ec19".to_owned(),
            ],
        },
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_network_interface() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot8").await;
    let steps = vec![];
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![],
        build_info(
            abi,
            8,
            vec![
                "0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3".to_owned(),
                "0xe38aed5884dc3e4426a87c083faaf4fa08109189fbc0c79281112f52e062d8ee".to_owned(),
                "0x442288826041d564ccedc579674f17c1b0a3452df799656a9167a41ab270ec19".to_owned(),
            ],
        ),
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_transaction_chain() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot9").await;
    let steps = serde_json::from_value(json!([])).unwrap();
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![format!("Test passed")],
        build_info(
            abi,
            9,
            vec!["0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3".to_owned()],
        ),
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_encryption_box() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot10").await;
    let steps = vec![];
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![format!("Encryption Box Handle: 3"), format!("Test passed")],
        build_info(
            abi,
            10,
            vec![
                format!("0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3"),
                format!("0x5b5f76b54d976d72f1ada3063d1af2e5352edaf1ba86b3b311170d4d81056d61"),
            ],
        ),
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_encryption_box_get_info() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot11").await;
    let steps = vec![];
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![],
        build_info(
            abi,
            11,
            vec![
                format!("0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3"),
                format!("0x5b5f76b54d976d72f1ada3063d1af2e5352edaf1ba86b3b311170d4d81056d61"),
            ],
        ),
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_signing_box_get_info() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot12").await;
    let steps = vec![];
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        steps,
        vec![],
        build_info(
            abi,
            12,
            vec![
                format!("0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3"),
                format!("0xc13024e101c95e71afb1f5fa6d72f633d51e721de0320d73dfd6121a54e4d40a"),
            ],
        ),
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_query() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot14").await;
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        vec![],
        vec![],
        build_info(
            abi,
            14,
            vec![
                format!("0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3"),
                format!("0x5c6fd81616cdfb963632109c42144a3a885c8d0f2e8deb5d8e15872fb92f2811"),
            ],
        ),
        vec![],
    )
    .await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_json_parse() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData {
        debot_addr,
        target_addr: _,
        keys,
        abi,
    } = init_simple_debot(client.clone(), "testDebot15").await;
    TestBrowser::execute_with_details(
        client.clone(),
        debot_addr.clone(),
        keys,
        vec![],
        vec![],
        build_info(
            abi,
            15,
            vec![
                format!("0x8796536366ee21852db56dccb60bc564598b618c865fc50c8b1ab740bba128e3"),
                format!("0x442288826041d564ccedc579674f17c1b0a3452df799656a9167a41ab270ec19"),
            ],
        ),
        vec![],
    )
    .await;
}

fn build_info(abi: String, n: u32, interfaces: Vec<String>) -> DebotInfo {
    let name = format!("TestDeBot{}", n);
    DebotInfo {
        name: Some(name.clone()),
        version: Some("0.1.0".to_owned()),
        publisher: Some("TON Labs".to_owned()),
        caption: Some(name.clone()),
        author: Some("TON Labs".to_owned()),
        support: Some(
            "0:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        ),
        hello: Some(name.clone()),
        language: Some("en".to_owned()),
        dabi: Some(abi),
        icon: Some(format!("")),
        interfaces,
    }
}

async fn download_account(client: &Arc<TestClient>, addr: &str) -> Option<String> {
    let client = client.clone();
    let accounts: ResultOfQueryCollection = client
        .request_async(
            "net.query_collection",
            ParamsOfQueryCollection {
                collection: format!("accounts"),
                filter: Some(json!({
                    "id": { "eq": addr }
                })),
                result: format!("boc"),
                limit: Some(1),
                order: None,
            },
        )
        .await
        .unwrap();

    if accounts.result.len() == 1 {
        Some(accounts.result[0]["boc"].as_str().unwrap().to_owned())
    } else {
        None
    }
}
async fn assert_get_method(
    client: &Arc<TestClient>,
    addr: &String,
    abi: &Abi,
    func: &str,
    params: Value,
    returns: Value,
) {
    let client = client.clone();
    let acc_boc = download_account(&client, &addr)
        .await
        .expect("Account not found");

    let call_params = ParamsOfEncodeMessage {
        abi: abi.clone(),
        deploy_set: None,
        signer: Signer::None,
        processing_try_index: None,
        address: Some(addr.clone()),
        call_set: CallSet::some_with_function_and_input(func, params),
    };

    let message = client.encode_message(call_params).await.unwrap().message;

    let result: ResultOfRunTvm = client
        .request_async(
            "tvm.run_tvm",
            ParamsOfRunTvm {
                account: acc_boc,
                message,
                abi: Some(abi.clone()),
                execution_options: None,
                boc_cache: None,
                return_updated_account: Some(true),
            },
        )
        .await
        .unwrap();

    let output = result.decoded.unwrap().output.expect("output must exist");
    assert_eq!(returns, output);
}
