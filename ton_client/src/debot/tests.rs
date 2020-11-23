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
use crate::abi::{CallSet, DeploySet, ParamsOfEncodeMessage, Signer};
use futures::future::{BoxFuture, FutureExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use super::*;

lazy_static!(
    static ref DEBOT: Mutex<Option<DebotData>> = Mutex::new(None);
);

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
}

impl TestBrowser {

    pub async fn execute_from_state(client: Arc<TestClient>, state: Arc<BrowserData>, start_function: &str) {
        let state_copy = state.clone();
        let client_copy = client.clone();
        let callback = move |params, response_type| {
            log::debug!("received from debot: {:#}", params);
            let client = client_copy.clone();
            let state = state_copy.clone();
            tokio::spawn(async move {
                match response_type {
                    ResponseType::AppNotify => {
                        Self::process_notification(&state, serde_json::from_value(params).unwrap()).await;
                    },
                    ResponseType::AppRequest => {
                        let request: ParamsOfAppRequest = serde_json::from_value(params).unwrap();
                        let result = Self::process_call(
                            client.clone(),
                            &state,
                            serde_json::from_value(request.request_data).unwrap()
                        ).await;
                        client.resolve_app_request(request.app_request_id, result).await;
                    },
                    _ => panic!("Wrong response type"),
                }
            });
            futures::future::ready(())
        };

        let handle: RegisteredDebot = client.request_async_callback(
            start_function,
            ParamsOfStart {
                address: state.address.clone()
            },
            callback
        ).await.unwrap();

        while !state.finished.load(Ordering::Relaxed) {
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
            assert_eq!(step.outputs, step.step.outputs);
            assert_eq!(step.step.inputs.len(), 0);
            assert_eq!(step.step.invokes.len(), 0);

            if step.available_actions.len() == 0 { break; }
        }

        assert_eq!(state.next.lock().await.len(), 0);
    }

    pub async fn execute(client: Arc<TestClient>, address: String, keys: KeyPair, steps: Vec<DebotStep>) {
        let state = Arc::new(BrowserData {
            current: Mutex::new(Default::default()), 
            next: Mutex::new(steps),
            client: client.clone(),
            keys,
            address: address.clone(),
            finished: AtomicBool::new(false),
        });

        Self::execute_from_state(client, state, "debot.start").await
    }

    async fn process_notification(state: &BrowserData, params: ParamsOfAppDebotBrowser) {
        match params {
            ParamsOfAppDebotBrowser::Log{ msg } => {
                state.current.lock().await.outputs.push(msg);
            },
            ParamsOfAppDebotBrowser::Switch { context_id } => {
                if context_id == STATE_EXIT {
                    state.finished.store(true, Ordering::Relaxed);
                }
                state.current.lock().await.available_actions.clear();
            },
            ParamsOfAppDebotBrowser::ShowAction { action } => {
                state.current.lock().await.available_actions.push(action);
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
            ParamsOfAppDebotBrowser::Input { prefix: _ } => {
                let value = state.current.lock().await.step.inputs.remove(0);
                ResultOfAppDebotBrowser::Input { value: value.to_owned() }
            },
            ParamsOfAppDebotBrowser::LoadKey => {
                let keys = state.keys.clone();
                ResultOfAppDebotBrowser::LoadKey { keys }
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
                });
                Self::call_execute_boxed(client, state, "debot.fetch").await;
                ResultOfAppDebotBrowser::InvokeDebot
            },
            _ => panic!("invalid call {:#?}", params)
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

    let target_addr = client.deploy_with_giver_async(ParamsOfEncodeMessage {
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
    ).await;

    let debot_addr = client.deploy_with_giver_async(ParamsOfEncodeMessage {
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
                    "debotAbi": hex::encode(&debot_abi.json_string().unwrap().as_bytes()),
                    "targetAbi": hex::encode(&target_abi.json_string().unwrap().as_bytes()),
                    "targetAddr": target_addr,
                }))
            }),
        },
        None
    ).await;

    let data = DebotData {
        debot_addr,
        target_addr,
        keys
    };
    *debot = Some(data.clone());
    data
}

#[tokio::test(core_threads = 2)]
async fn test_debot_goto() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 1, "inputs": [], "outputs": ["Test Goto Action"] },
        { "choice": 1, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": 8, "inputs": [], "outputs": [] }
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
        { "choice": 8, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
    ).await;
}

#[tokio::test(core_threads = 2)]
async fn test_debot_run() {
    let client = std::sync::Arc::new(TestClient::new());
    let DebotData { debot_addr, target_addr: _, keys } = init_debot(client.clone()).await;

    let steps = json!([
        { "choice": 3, "inputs": ["-1:1111111111111111111111111111111111111111111111111111111111111111"], "outputs": ["Test Run Action", "test1: instant run 1", "test2: instant run 2"] },
        { "choice": 1, "inputs": ["hello"], "outputs": [] },
        { "choice": 2, "inputs": [], "outputs": [ "integer=2,addr=-1:1111111111111111111111111111111111111111111111111111111111111111,string=hello"] },
        { "choice": 3, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": 8, "inputs": [], "outputs": [] },
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
        { "choice": 8, "inputs": [], "outputs": [] },
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
        { "choice": 1, "inputs": [], "outputs": ["Transaction succeeded."] },
        { "choice": 2, "inputs": [], "outputs": ["Debot Tests"] },
        { "choice": 8, "inputs": [], "outputs": [] },
    ]);
    TestBrowser::execute(
        client.clone(),
        debot_addr.clone(),
        keys.clone(),
        serde_json::from_value(steps).unwrap()
    ).await;
}
