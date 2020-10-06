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

use super::InteropContext;
use super::{tc_destroy_json_response, tc_read_json_response};
use super::{tc_json_request, tc_json_request_sync, InteropString};
use crate::client::{ClientContext, ParamsOfUnregisterCallback, ResponseType};
use crate::crypto::{
    ParamsOfNaclSignDetached, ParamsOfNaclSignKeyPairFromSecret, ResultOfNaclSignDetached,
};
use crate::{
    client::ResultOfCreateContext,
    contracts::{
        deploy::{ParamsOfDeploy, ResultOfDeploy},
        run::{ParamsOfRun, ResultOfRun, RunFunctionCallSet},
        EncodedMessage,
    },
    crypto::KeyPair,
    dispatch::Callback,
    error::{ApiError, ApiResult},
    net::{ParamsOfWaitForCollection, ResultOfWaitForCollection},
    tc_create_context, tc_destroy_context, JsonResponse,
};
use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::{
    Mutex,
    mpsc::{channel, Sender}
};
use num_traits::FromPrimitive;

mod common;

const ROOT_CONTRACTS_PATH: &str = "src/tests/contracts/";
const LOG_CGF_PATH: &str = "src/tests/log_cfg.yaml";

struct SimpleLogger;

const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Warn;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            log::Level::Error | log::Level::Warn => {
                eprintln!("{}", record.args());
            }
            _ => {
                println!("{}", record.args());
            }
        }
    }

    fn flush(&self) {}
}

// pub const SUBSCRIBE: &str = "Subscription";
// pub const PIGGY_BANK: &str = "Piggy";
// pub const WALLET: &str = "LimitWallet";
// pub const SIMPLE_WALLET: &str = "Wallet";
pub const GIVER: &str = "Giver";
pub const GIVER_WALLET: &str = "GiverWallet";
pub const HELLO: &str = "Hello";
pub const EVENTS: &str = "Events";

struct RequestData {
    sender: Sender<JsonResponse>,
    callback: Box<dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>
}

struct TestRuntime {
    pub next_request_id: u32,
    pub next_callback_id: u32,
    pub requests: HashMap<u32, RequestData>,
}

impl TestRuntime {
    fn new() -> Self {
        Self {
            next_callback_id: 1,
            next_request_id: 1,
            requests: HashMap::new(),
        }
    }

    fn gen_request_id(&mut self) -> u32 {
        let id = self.next_request_id;
        self.next_request_id += 1;
        id
    }
}

lazy_static::lazy_static! {
    static ref TEST_RUNTIME: Mutex<TestRuntime> = Mutex::new(TestRuntime::new());
}

#[derive(Clone)]
pub(crate) struct TestClient {
    context: InteropContext,
}

extern "C" fn on_result(
    request_id: u32,
    params_json: InteropString,
    response_type: u32,
    finished: bool
) {
    TestClient::on_result(request_id, params_json.to_string(), response_type, finished)
}

pub struct AsyncFuncWrapper<'a, P, R> {
    client: &'a TestClient,
    name: String,
    p: std::marker::PhantomData<(P, R)>,
}

impl<'a, P: Serialize, R: DeserializeOwned> AsyncFuncWrapper<'a, P, R> {
    pub(crate) async fn call(&self, params: P) -> R {
        self.client.request_async(&self.name, params).await
    }

    pub(crate) async fn call_with_callback<CF, CT, CR>(
        &self,
        params: P,
        callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static
    ) -> R 
    where 
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned
    {
        self.client.request_async_callback(&self.name, params, callback).await
    }
}

impl TestClient {
    pub(crate) fn wrap_async<P, R, F>(
        self: &TestClient,
        _: fn(Arc<ClientContext>, P) -> F,
        info: fn() -> api_doc::api::Method,
    ) -> AsyncFuncWrapper<P, R>
    where
        P: Serialize,
        R: DeserializeOwned,
        F: Future<Output = ApiResult<R>>,
    {
        AsyncFuncWrapper {
            client: self,
            name: info().name,
            p: std::marker::PhantomData::default(),
        }
    }

    pub(crate) fn wrap_async_callback<P, R, F>(
        self: &TestClient,
        _: fn(Arc<ClientContext>, P, std::sync::Arc<Callback>) -> F,
        info: fn() -> api_doc::api::Method,
    ) -> AsyncFuncWrapper<P, R>
    where
        P: Serialize,
        R: DeserializeOwned,
        F: Future<Output = ApiResult<R>>,
    {
        AsyncFuncWrapper {
            client: self,
            name: info().name,
            p: std::marker::PhantomData::default(),
        }
    }

    pub(crate) fn wrap<P, R>(
        self: &TestClient,
        _: fn(Arc<ClientContext>, P) -> ApiResult<R>,
        info: fn() -> api_doc::api::Method,
    ) -> AsyncFuncWrapper<P, R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        AsyncFuncWrapper {
            client: self,
            name: info().name,
            p: std::marker::PhantomData::default(),
        }
    }

    fn read_abi(path: String) -> Value {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    pub fn giver_address() -> String {
        "0:841288ed3b55d9cdafa806807f02a0ae0c169aa5edfe88a789a6482429756a94".into()
    }

    pub fn giver_abi() -> Value {
        if Self::node_se() {
            Self::abi(GIVER, Some(1))
        } else {
            Self::abi(GIVER_WALLET, Some(2))
        }
    }

    pub fn wallet_address() -> String {
        "0:2bb4a0e8391e7ea8877f4825064924bd41ce110fce97e939d3323999e1efbb13".into()
    }

    pub fn wallet_keys() -> Option<KeyPair> {
        if Self::node_se() {
            return None;
        }

        let mut keys_file = dirs::home_dir().unwrap();
        keys_file.push("giverKeys.json");
        let keys = std::fs::read_to_string(keys_file).unwrap();

        Some(serde_json::from_str(&keys).unwrap())
    }

    pub fn network_address() -> String {
        std::env::var("TON_NETWORK_ADDRESS")
            .unwrap_or("http://localhost".to_owned())
            //.unwrap_or("cinet.tonlabs.io".to_owned())
            //.unwrap_or("net.ton.dev".to_owned())
    }

    pub fn node_se() -> bool {
        std::env::var("USE_NODE_SE").unwrap_or("true".to_owned()) == "true".to_owned()
    }

    pub fn abi_version() -> u8 {
        u8::from_str_radix(&std::env::var("ABI_VERSION").unwrap_or("2".to_owned()), 10).unwrap()
    }

    pub fn contracts_path(abi_version: Option<u8>) -> String {
        format!(
            "{}abi_v{}/",
            ROOT_CONTRACTS_PATH,
            abi_version.unwrap_or(Self::abi_version())
        )
    }

    pub fn abi(name: &str, version: Option<u8>) -> Value {
        Self::read_abi(format!(
            "{}{}.abi.json",
            Self::contracts_path(version),
            name
        ))
    }

    pub fn tvc(name: &str, abi_version: Option<u8>) -> String {
        base64::encode(
            &std::fs::read(format!("{}{}.tvc", Self::contracts_path(abi_version), name)).unwrap(),
        )
    }

    pub fn package(name: &str, abi_version: Option<u8>) -> (Value, String) {
        (Self::abi(name, abi_version), Self::tvc(name, abi_version))
    }

    pub(crate) fn init_log() {
        let log_cfg_path = LOG_CGF_PATH;
        let _ = log4rs::init_file(log_cfg_path, Default::default());
    }

    pub(crate) fn new() -> Self {
        Self::new_with_config(json!({
            "network": {
                "server_address": Self::network_address()
            }
        }))
    }

    pub(crate) fn new_with_config(config: Value) -> Self {
        let _ =
            log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(MAX_LEVEL));

        let response = unsafe {
            let response_ptr = tc_create_context(InteropString::from(&config.to_string()));
            let interop_response = tc_read_json_response(response_ptr);
            let response = interop_response.to_response();
            tc_destroy_json_response(response_ptr);
            response
        };

        let context = if response.error_json.is_empty() {
            let result: ResultOfCreateContext =
                serde_json::from_str(&response.result_json).unwrap();
            result.handle
        } else {
            panic!("tc_create_context returned error: {}", response.error_json);
        };

        let client = Self { context };
        client
    }

    fn response_to_result(response: JsonResponse) -> ApiResult<Value> {
        if response.error_json.is_empty() {
            if response.result_json.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(serde_json::from_str(&response.result_json).unwrap())
            }
        } else {
            println!("Error {}", response.error_json);
            Err(serde_json::from_str(&response.error_json).unwrap())
        }
    }

    pub(crate) fn request_json(&self, method: &str, params: Value) -> ApiResult<Value> {
        Self::response_to_result(unsafe {
            let params_json = if params.is_null() {
                String::new()
            } else {
                params.to_string()
            };
            let response_ptr = tc_json_request_sync(
                self.context,
                InteropString::from(&method.to_string()),
                InteropString::from(&params_json),
            );
            let interop_response = tc_read_json_response(response_ptr);
            let response = interop_response.to_response();
            tc_destroy_json_response(response_ptr);
            response
        })
    }

    pub(crate) fn request<P, R>(&self, method: &str, params: P) -> R
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let params = serde_json::to_value(params).unwrap();
        let result = self.request_json(method, params).unwrap();
        serde_json::from_value(result).unwrap()
    }

    fn on_result(
        request_id: u32,
        params_json: String,
        response_type: u32,
        finished: bool,
    ) {
        // we have to process callback in another thread because:
        // 1. processing must be async because sender which resolves funtion result is async
        // 2. `rt_handle.enter` function processes task in backgroud without ability to wait for its completion.
        //  But we need to preserve the order of `on_result` calls processing, otherwise call with 
        //  `finished` = true can be processed before previous call and remove callback handler
        //  while it's still needed
        // 3. `rt_handle.block_on` function can't be used in current thread because thread is in async 
        //  context so we have spawn antoher thread and use `rt_handle.block_on` function there
        //  and then wait for thread completion
        let rt_handle = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            rt_handle.block_on(
                Self::on_result_async(request_id, params_json.to_string(), response_type, finished)
            );
        })
        .join()
        .unwrap();
    }

    async fn on_result_async(
        request_id: u32,
        params_json: String,
        response_type: u32,
        finished: bool,
    ) {
        log::debug!("on_result response-type: {} params_json: {}", response_type, params_json);
        let requests =  &mut TEST_RUNTIME
            .lock()
            .await
            .requests;
        let request = requests
            .get_mut(&request_id)
            .unwrap();

        match ResponseType::from_u32(response_type) {
            Some(std_response_type) => {
                match std_response_type {
                    ResponseType::Success => {
                        request.sender.send(JsonResponse {
                            result_json: params_json.to_string(),
                            error_json: String::new(),
                        })
                        .await
                        .unwrap();
                    },
                    ResponseType::Error => {
                        request.sender.send(JsonResponse {
                            result_json: String::new(),
                            error_json: params_json.to_string(),
                        })
                        .await
                        .unwrap();
                    },
                    _ => {}
                };
            },
            None => {
                (request.callback)(params_json.to_string(), response_type).await
            }
        }

        if finished {
            requests.remove(&request_id);
        }
    }

    pub(crate) async fn request_json_async_callback<CR, CT, CF>(
        &self, method: &str, params: Value, callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static
    ) -> ApiResult<Value>
    where 
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned
     {
        let callback = move |params_json: String, response_type: u32| {
            let params: CR = serde_json::from_str(&params_json).unwrap();
            let response_type = CT::from_u32(response_type).unwrap();
            Box::pin(callback(params, response_type)) as Pin<Box<dyn Future<Output = ()> + Send + Sync>>
        };
        //let callback = Box::new(callback);
        let (request_id, mut receiver) = {
            let mut runtime = TEST_RUNTIME.lock().await;
            let id = runtime.gen_request_id();
            let (sender, receiver) = channel(10);
            runtime.requests.insert(id, RequestData {
                sender,
                callback: Box::new(callback)// as Box<dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>
            });
            (id, receiver)
        };
        unsafe {
            let params_json = if params.is_null() {
                String::new()
            } else {
                params.to_string()
            };
            tc_json_request(
                self.context,
                InteropString::from(&method.to_string()),
                InteropString::from(&params_json),
                request_id,
                on_result,
            );
        };
        Self::response_to_result(receiver.recv().await.unwrap())
    }

    pub(crate) async fn request_async_callback<P, R, CR, CT, CF>(
        &self, method: &str, params: P, callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static
    ) -> R
    where
        P: Serialize,
        R: DeserializeOwned,
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned
    {
        let params = serde_json::to_value(params).unwrap();
        let result = self.request_json_async_callback(method, params, callback).await.unwrap();
        serde_json::from_value(result).unwrap()
    }

    pub(crate) async fn request_json_async(&self, method: &str, params: Value) -> ApiResult<Value> {
        let callback = |_: Value, _: u32| async { panic!("wrong response type") };
        self.request_json_async_callback(method, params, callback).await
    }

    pub(crate) async fn request_async<P, R>(&self, method: &str, params: P) -> R
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let callback = |_: Value, _: u32| async { panic!("wrong response type") };
        self.request_async_callback(method, params, callback).await
    }

    pub(crate) fn request_no_params<R: DeserializeOwned>(&self, method: &str) -> R {
        let result = self.request_json(method, Value::Null).unwrap();
        serde_json::from_value(result)
            .map_err(|err| ApiError::invalid_params("", err))
            .unwrap()
    }

    pub(crate) fn get_grams_from_giver(&self, account: &str, value: Option<u64>) {
        let run_result: ResultOfRun = if Self::node_se() {
            self.request(
                "contracts.run",
                ParamsOfRun {
                    address: Self::giver_address().into(),
                    call_set: RunFunctionCallSet {
                        abi: Self::giver_abi(),
                        function_name: "sendGrams".to_owned(),
                        header: None,
                        input: json!({
                            "dest": account,
                            "amount": value.unwrap_or(500_000_000u64)
                        }),
                    },
                    key_pair: None,
                    try_index: None,
                },
            )
        } else {
            self.request(
                "contracts.run",
                ParamsOfRun {
                    address: Self::wallet_address().into(),
                    call_set: RunFunctionCallSet {
                        abi: Self::giver_abi(),
                        function_name: "sendTransaction".to_owned(),
                        header: None,
                        input: json!({
                            "dest": account.to_string(),
                            "value": value.unwrap_or(500_000_000u64),
                            "bounce": false
                        }),
                    },
                    key_pair: Self::wallet_keys(),
                    try_index: None,
                },
            )
        };

        // wait for grams recieving
        for message in run_result.transaction["out_messages"].as_array().unwrap() {
            let message: ton_sdk::Message = serde_json::from_value(message.clone()).unwrap();
            if ton_sdk::MessageType::Internal == message.msg_type() {
                let _: ResultOfWaitForCollection = self.request(
                    "net.wait_for_collection",
                    ParamsOfWaitForCollection {
                        collection: "transactions".to_owned(),
                        filter: Some(json!({
                            "in_msg": { "eq": message.id()}
                        })),
                        result: "id".to_owned(),
                        timeout: Some(ton_sdk::types::DEFAULT_WAIT_TIMEOUT),
                    },
                );
            }
        }
    }

    pub(crate) async fn get_grams_from_giver_async(&self, account: &str, value: Option<u64>) {
        let run_result: ResultOfRun = if Self::node_se() {
            self.request_async(
                "contracts.run",
                ParamsOfRun {
                    address: Self::giver_address().into(),
                    call_set: RunFunctionCallSet {
                        abi: Self::giver_abi(),
                        function_name: "sendGrams".to_owned(),
                        header: None,
                        input: json!({
                            "dest": account,
                            "amount": value.unwrap_or(500_000_000u64)
                        }),
                    },
                    key_pair: None,
                    try_index: None,
                },
            )
            .await
        } else {
            self.request_async(
                "contracts.run",
                ParamsOfRun {
                    address: Self::wallet_address().into(),
                    call_set: RunFunctionCallSet {
                        abi: Self::giver_abi(),
                        function_name: "sendTransaction".to_owned(),
                        header: None,
                        input: json!({
                            "dest": account.to_string(),
                            "value": value.unwrap_or(500_000_000u64),
                            "bounce": false
                        }),
                    },
                    key_pair: Self::wallet_keys(),
                    try_index: None,
                },
            )
            .await
        };

        // wait for grams recieving
        for message in run_result.transaction["out_messages"].as_array().unwrap() {
            let message: ton_sdk::Message = serde_json::from_value(message.clone()).unwrap();
            if ton_sdk::MessageType::Internal == message.msg_type() {
                let _: ResultOfWaitForCollection = self
                    .request_async(
                        "net.wait_for_collection",
                        ParamsOfWaitForCollection {
                            collection: "transactions".to_owned(),
                            filter: Some(json!({
                                "in_msg": { "eq": message.id()}
                            })),
                            result: "id".to_owned(),
                            timeout: Some(ton_sdk::types::DEFAULT_WAIT_TIMEOUT),
                        },
                    )
                    .await;
            }
        }
    }

    pub(crate) fn deploy_with_giver(&self, params: ParamsOfDeploy, value: Option<u64>) -> String {
        let msg: EncodedMessage = self.request("contracts.deploy.message", params.clone());

        self.get_grams_from_giver(&msg.address.unwrap(), value);

        let result: ResultOfDeploy = self.request("contracts.deploy", params);

        result.address
    }

    pub(crate) async fn deploy_with_giver_async(
        &self,
        params: ParamsOfDeploy,
        value: Option<u64>,
    ) -> String {
        let msg: EncodedMessage = self
            .request_async("contracts.deploy.message", params.clone())
            .await;

        self.get_grams_from_giver_async(&msg.address.unwrap(), value)
            .await;

        let result: ResultOfDeploy = self.request_async("contracts.deploy", params).await;

        result.address
    }

    pub(crate) fn generate_sign_keys(&self) -> KeyPair {
        self.request("crypto.generate_random_sign_keys", ())
    }

    pub fn sign_detached(&self, data: &str, keys: &KeyPair) -> String {
        let sign_keys: KeyPair = self.request(
            "crypto.nacl_sign_keypair_from_secret",
            ParamsOfNaclSignKeyPairFromSecret {
                secret: keys.secret.clone(),
            },
        );
        let result: ResultOfNaclSignDetached = self.request(
            "crypto.nacl_sign_detached",
            ParamsOfNaclSignDetached {
                unsigned: data.into(),
                secret: sign_keys.secret.clone(),
            },
        );
        result.signature
    }

    pub(crate) fn get_giver_address() -> String {
        if Self::node_se() {
            Self::giver_address()
        } else {
            Self::wallet_address()
        }
        .into()
    }
}

impl Drop for TestClient {
    fn drop(&mut self) {
        unsafe {
            if self.context != 0 {
                tc_destroy_context(self.context)
            }
        }
    }
}
