/*
* Copyright 2018-2021 TON Labs LTD.
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

use super::{tc_destroy_string, tc_read_string, tc_request, tc_request_sync};
use crate::abi::{
    encode_message, Abi, CallSet, DeploySet, ParamsOfEncodeMessage, ResultOfEncodeMessage, Signer,
};
use crate::client::*;
use crate::crypto::internal::hex_decode_secret_const;
use crate::crypto::mnemonic::ed25519_keys_from_secret_bytes;
use crate::crypto::{
    ParamsOfNaclSignDetached, ParamsOfNaclSignKeyPairFromSecret, ResultOfNaclSignDetached,
};
use crate::json_interface::interop::{ResponseType, StringData};
use crate::json_interface::modules::{AbiModule, ProcessingModule};
use crate::json_interface::runtime::Runtime;
use crate::net::{ParamsOfQuery, ResultOfQuery};
use crate::processing::{ParamsOfProcessMessage, ResultOfProcessMessage};
use crate::{
    crypto::KeyPair,
    error::{ClientError, ClientResult},
    net::{ParamsOfQueryTransactionTree, ResultOfQueryTransactionTree},
    tc_create_context, tc_destroy_context, ContextHandle,
};
use api_info::ApiModule;
use futures::Future;
use num_traits::FromPrimitive;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{
    oneshot::{channel, Sender},
    Mutex,
};

mod common;

const ROOT_CONTRACTS_PATH: &str = "src/tests/contracts/";
const LOG_CGF_PATH: &str = "src/tests/log_cfg.yaml";

mod env {
    fn str(name: &str, alt: Option<&str>) -> Option<String> {
        if let Ok(s) = std::env::var(name) {
            Some(s)
        } else if let Some(alt) = alt {
            std::env::var(alt).ok()
        } else {
            None
        }
    }

    pub(crate) fn giver_address() -> Option<String> {
        str("EVERCLOUD_GIVER_ADDRESS", Some("TON_GIVER_ADDRESS"))
    }

    pub(crate) fn giver_secret() -> Option<String> {
        str("EVERCLOUD_GIVER_SECRET", Some("TON_GIVER_SECRET"))
    }

    pub(crate) fn giver_type() -> String {
        str("EVERCLOUD_GIVER_TYPE", None).unwrap_or_else(|| "v2".into())
    }

    pub(crate) fn auth_project() -> Option<String> {
        str("EVERCLOUD_AUTH_PROJECT", None)
    }

    pub(crate) fn endpoints() -> String {
        str("EVERCLOUD_ENDPOINTS", Some("TON_NETWORK_ADDRESS"))
            .unwrap_or_else(|| "http://localhost".into())
    }

    pub(crate) fn queries_protocol() -> String {
        str("EVERCLOUD_QUERIES_PROTOCOL", Some("TON_QUERIES_PROTOCOL")).unwrap_or_else(|| "".into())
    }

    pub(crate) fn node_se() -> String {
        str("EVERCLOUD_NODE_SE", Some("TON_USE_SE")).unwrap_or_else(|| "true".into())
    }

    pub(crate) fn abi_version() -> String {
        str("EVERCLOUD_ABI_VERSION", Some("ABI_VERSION")).unwrap_or_else(|| "2".into())
    }
}

struct SimpleLogger;

const MAX_LEVEL: log::LevelFilter = log::LevelFilter::Warn;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() < MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        println!(
            "{} {}",
            chrono::prelude::Utc::now().timestamp_millis(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub const SUBSCRIBE: &str = "Subscription";
// pub const PIGGY_BANK: &str = "Piggy";
// pub const WALLET: &str = "LimitWallet";
// pub const SIMPLE_WALLET: &str = "Wallet";
pub const GIVER_V3: &str = "GiverV3";
pub const GIVER_V2: &str = "GiverV2";
pub const HELLO: &str = "Hello";
pub const EVENTS_OLD: &str = "EventsOld";
pub const EVENTS: &str = "Events";
pub const T24_INIT_DATA: &str = "t24_initdata";
pub const TEST_DEBOT: &str = "testDebot";
pub const TEST_DEBOT_TARGET: &str = "testDebotTarget";
pub const EXCEPTION: &str = "Exception";

struct RequestData {
    sender: Option<Sender<ClientResult<Value>>>,
    callback:
        Box<dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>,
}

struct TestRuntime {
    pub next_request_id: u32,
    pub requests: HashMap<u32, RequestData>,
}

impl TestRuntime {
    fn new() -> Self {
        Self {
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

pub(crate) struct TestClient {
    context: ContextHandle,
}

extern "C" fn on_result(
    request_id: u32,
    params_json: StringData,
    response_type: u32,
    finished: bool,
) {
    TestClient::on_result(request_id, params_json.to_string(), response_type, finished)
}

pub struct AsyncFuncWrapper<'a, P, R> {
    client: &'a TestClient,
    name: String,
    p: std::marker::PhantomData<(P, R)>,
}

impl<'a, P: Serialize, R: DeserializeOwned> AsyncFuncWrapper<'a, P, R> {
    pub(crate) async fn call(&self, params: P) -> ClientResult<R> {
        self.client.request_async(&self.name, params).await
    }

    pub(crate) async fn call_with_callback<CF, CT, CR>(
        &self,
        params: P,
        callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static,
    ) -> ClientResult<R>
    where
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned,
    {
        self.client
            .request_async_callback(&self.name, params, callback)
            .await
    }
}

pub struct FuncWrapper<'a, P, R> {
    client: &'a TestClient,
    name: String,
    p: std::marker::PhantomData<(P, R)>,
}

impl<'a, P: Serialize, R: DeserializeOwned> FuncWrapper<'a, P, R> {
    pub(crate) fn call(&self, params: P) -> ClientResult<R> {
        self.client.request(&self.name, params)
    }
}

fn parse_sync_response<R: DeserializeOwned>(response: *const String) -> ClientResult<R> {
    let response = unsafe {
        let result = tc_read_string(response).to_string();
        tc_destroy_string(response);
        result
    };
    match serde_json::from_str::<Value>(&response) {
        Ok(value) => {
            if value["error"].is_object() {
                Err(serde_json::from_value::<ClientError>(value["error"].clone()).unwrap())
            } else {
                Ok(serde_json::from_value(value["result"].clone()).unwrap())
            }
        }
        Err(err) => Err(Error::cannot_serialize_result(err)),
    }
}

impl TestClient {
    pub(crate) fn wrap_async<P, R, F>(
        self: &TestClient,
        _: fn(Arc<ClientContext>, P) -> F,
        module: api_info::Module,
        function: api_info::Function,
    ) -> AsyncFuncWrapper<P, R>
    where
        P: Serialize,
        R: DeserializeOwned,
        F: Future<Output = ClientResult<R>>,
    {
        AsyncFuncWrapper {
            client: self,
            name: format!("{}.{}", module.name, function.name),
            p: std::marker::PhantomData::default(),
        }
    }

    pub(crate) fn wrap_async_callback<P, R, F>(
        self: &TestClient,
        _: fn(Arc<ClientContext>, P, Arc<crate::json_interface::request::Request>) -> F,
        module: api_info::Module,
        function: api_info::Function,
    ) -> AsyncFuncWrapper<P, R>
    where
        P: Serialize,
        R: DeserializeOwned,
        F: Future<Output = ClientResult<R>>,
    {
        AsyncFuncWrapper {
            client: self,
            name: format!("{}.{}", module.name, function.name),
            p: std::marker::PhantomData::default(),
        }
    }

    pub(crate) fn wrap<P, R>(
        self: &TestClient,
        _: fn(Arc<ClientContext>, P) -> ClientResult<R>,
        module: api_info::Module,
        function: api_info::Function,
    ) -> FuncWrapper<P, R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        FuncWrapper {
            client: self,
            name: format!("{}.{}", module.name, function.name),
            p: std::marker::PhantomData::default(),
        }
    }

    pub fn read_abi(path: String) -> Abi {
        Abi::Contract(serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
    }

    pub fn giver_abi() -> Abi {
        match env::giver_type().as_str() {
            "v3" => Self::abi(GIVER_V3, Some(2)),
            "v2" => Self::abi(GIVER_V2, Some(2)),
            "v1" => Self::abi("Giver", Some(1)),
            _ => panic!("Unknown giver type"),
        }
    }

    pub fn giver_tvc() -> Option<String> {
        match env::giver_type().as_str() {
            "v3" => Self::tvc(GIVER_V3, Some(2)),
            "v2" => Self::tvc(GIVER_V2, Some(2)),
            "v1" => Self::tvc("Giver", Some(1)),
            _ => panic!("Unknown giver type"),
        }
    }

    async fn calc_giver_address(&self, keys: KeyPair) -> String {
        self.encode_message(ParamsOfEncodeMessage {
            abi: Self::giver_abi(),
            deploy_set: DeploySet::some_with_tvc(Self::giver_tvc()),
            signer: Signer::Keys { keys },
            ..Default::default()
        })
        .await
        .unwrap()
        .address
    }

    pub async fn giver_address(&self) -> String {
        if let Some(address) = env::giver_address() {
            address
        } else {
            self.calc_giver_address(Self::giver_keys()).await
        }
    }

    pub fn giver_keys() -> KeyPair {
        if let Some(secret) = env::giver_secret() {
            ed25519_keys_from_secret_bytes(&hex_decode_secret_const(&secret).unwrap().0).unwrap()
        } else {
            KeyPair {
                public: "2ada2e65ab8eeab09490e3521415f45b6e42df9c760a639bcf53957550b25a16"
                    .to_owned(),
                secret: "172af540e43a524763dd53b26a066d472a97c4de37d5498170564510608250c3"
                    .to_owned(),
            }
        }
    }

    pub fn auth_project() -> Option<String> {
        env::auth_project()
    }

    pub fn with_project(endpoint: &str) -> String {
        match Self::auth_project() {
            Some(project) => {
                if endpoint.ends_with('/') {
                    format!("{}{}", endpoint, project)
                } else {
                    format!("{}/{}", endpoint, project)
                }
            }
            None => endpoint.to_string(),
        }
    }

    pub fn endpoints() -> Vec<String> {
        env::endpoints()
            .split(",")
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect()
    }

    pub fn queries_protocol() -> Option<String> {
        let protocol = env::queries_protocol().trim().to_string();
        if protocol.is_empty() {
            None
        } else {
            Some(protocol)
        }
    }

    pub fn node_se() -> bool {
        env::node_se() == "true"
    }

    pub fn abi_version() -> u8 {
        u8::from_str_radix(&env::abi_version(), 10).unwrap()
    }

    pub fn contracts_path(abi_version: Option<u8>) -> String {
        format!(
            "{}abi_v{}/",
            ROOT_CONTRACTS_PATH,
            abi_version.unwrap_or(Self::abi_version())
        )
    }

    pub fn abi(name: &str, version: Option<u8>) -> Abi {
        Self::read_abi(format!(
            "{}{}.abi.json",
            Self::contracts_path(version),
            name
        ))
    }

    pub fn tvc(name: &str, abi_version: Option<u8>) -> Option<String> {
        Some(base64::encode(
            &std::fs::read(format!("{}{}.tvc", Self::contracts_path(abi_version), name)).unwrap(),
        ))
    }

    pub fn icon(name: &str, abi_version: Option<u8>) -> String {
        let image_base64 = base64::encode(
            &std::fs::read(format!("{}{}.png", Self::contracts_path(abi_version), name)).unwrap(),
        );
        format!("data:image/png;base64,{}", image_base64)
    }

    pub fn package(name: &str, abi_version: Option<u8>) -> (Abi, Option<String>) {
        (Self::abi(name, abi_version), Self::tvc(name, abi_version))
    }

    pub fn init_simple_logger() {
        let _ =
            log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(MAX_LEVEL));
    }

    pub(crate) fn init_log() {
        let log_cfg_path = LOG_CGF_PATH;
        if log4rs::init_file(log_cfg_path, Default::default()).is_err() {
            Self::init_simple_logger();
        }
    }

    pub(crate) fn new() -> Self {
        Self::new_with_config(json!({
            "network": {
                "endpoints": TestClient::endpoints(),
                "queries_protocol": TestClient::queries_protocol(),
            }
        }))
    }

    pub(crate) fn new_with_config(config: Value) -> Self {
        Self::init_simple_logger();

        unsafe {
            let response = tc_create_context(StringData::new(&config.to_string()));
            Self {
                context: parse_sync_response(response).unwrap(),
            }
        }
    }

    pub(crate) fn request_json(&self, method: &str, params: Value) -> ClientResult<Value> {
        let params_json = if params.is_null() {
            String::new()
        } else {
            params.to_string()
        };
        parse_sync_response(unsafe {
            tc_request_sync(
                self.context,
                StringData::new(&method.to_string()),
                StringData::new(&params_json),
            )
        })
    }

    pub(crate) fn request<P, R>(&self, method: &str, params: P) -> ClientResult<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let params = serde_json::to_value(params).unwrap();
        self.request_json(method, params)
            .map(|result| serde_json::from_value(result).unwrap())
    }

    fn on_result(request_id: u32, params_json: String, response_type: u32, finished: bool) {
        // we have to process callback in another thread because:
        // 1. processing must be async because sender which resolves function result is async
        // 2. `rt_handle.enter` function processes task in background without ability to wait for its completion.
        //  But we need to preserve the order of `on_result` calls processing, otherwise call with
        //  `finished` = true can be processed before previous call and remove callback handler
        //  while it's still needed
        // 3. `rt_handle.block_on` function can't be used in current thread because thread is in async
        //  context so we have to spawn another thread and use `rt_handle.block_on` function there
        //  and then wait for thread completion
        let rt_handle = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            rt_handle.block_on(Self::on_result_async(
                request_id,
                params_json.to_string(),
                response_type,
                finished,
            ));
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
        //log::debug!("on_result response-type: {} params_json: {}", response_type, params_json);
        let requests = &mut TEST_RUNTIME.lock().await.requests;
        let request = requests.get_mut(&request_id).unwrap();

        if response_type == ResponseType::Success as u32 {
            request
                .sender
                .take()
                .unwrap()
                .send(Ok(serde_json::from_str::<Value>(&params_json).unwrap()))
                .unwrap();
        } else if response_type == ResponseType::Error as u32 {
            let err = match serde_json::from_str::<ClientError>(&params_json) {
                Ok(err) => err,
                Err(err) => Error::callback_params_cant_be_converted_to_json(err),
            };
            request.sender.take().unwrap().send(Err(err)).unwrap();
        } else if response_type == ResponseType::Nop as u32 {
        } else if response_type >= ResponseType::Custom as u32
            || response_type == ResponseType::AppRequest as u32
            || response_type == ResponseType::AppNotify as u32
        {
            (request.callback)(params_json, response_type).await
        } else {
            panic!("Unsupported response type: {}", response_type);
        }

        if finished {
            requests.remove(&request_id);
        }
    }

    pub(crate) async fn request_json_async_callback<CR, CT, CF>(
        &self,
        method: &str,
        params: Value,
        callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static,
    ) -> ClientResult<Value>
    where
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned,
    {
        let callback = move |params_json: String, response_type: u32| {
            let params: CR = serde_json::from_str(&params_json).unwrap();
            let response_type = CT::from_u32(response_type).unwrap();
            Box::pin(callback(params, response_type))
                as Pin<Box<dyn Future<Output = ()> + Send + Sync>>
        };
        //let callback = Box::new(callback);
        let (request_id, receiver) = {
            let mut runtime = TEST_RUNTIME.lock().await;
            let id = runtime.gen_request_id();
            let (sender, receiver) = channel();
            runtime.requests.insert(
                id,
                RequestData {
                    sender: Some(sender),
                    callback: Box::new(callback), // as Box<dyn Fn(String, u32) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>
                },
            );
            (id, receiver)
        };
        unsafe {
            let params_json = if params.is_null() {
                String::new()
            } else {
                params.to_string()
            };
            tc_request(
                self.context,
                StringData::new(&method.to_string()),
                StringData::new(&params_json),
                request_id,
                on_result,
            );
        };
        let response = receiver.await.unwrap();
        response
    }

    pub(crate) async fn request_async_callback<P, R, CR, CT, CF>(
        &self,
        method: &str,
        params: P,
        callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static,
    ) -> ClientResult<R>
    where
        P: Serialize,
        R: DeserializeOwned,
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned,
    {
        let params = serde_json::to_value(params).unwrap();
        self.request_json_async_callback(method, params, callback)
            .await
            .map(|result| serde_json::from_value(result).unwrap())
    }

    pub async fn default_callback(_: Value, _: u32) {
        panic!("wrong response type");
    }

    pub(crate) async fn request_async<P, R>(&self, method: &str, params: P) -> ClientResult<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        self.request_async_callback(method, params, Self::default_callback)
            .await
    }

    pub(crate) fn request_no_params<R: DeserializeOwned>(&self, method: &str) -> ClientResult<R> {
        self.request_json(method, Value::Null)
            .map(|result| serde_json::from_value(result).unwrap())
    }

    pub(crate) async fn encode_message(
        &self,
        params: ParamsOfEncodeMessage,
    ) -> ClientResult<ResultOfEncodeMessage> {
        let encode = self.wrap_async(
            encode_message,
            AbiModule::api(),
            crate::abi::encode_message::encode_message_api(),
        );
        encode.call(params).await
    }

    pub(crate) fn encode_message_sync(
        &self,
        params: ParamsOfEncodeMessage,
    ) -> ClientResult<ResultOfEncodeMessage> {
        self.request("abi.encode_message", params)
    }

    pub(crate) async fn net_process_message<CF, CT, CR>(
        &self,
        params: ParamsOfProcessMessage,
        callback: impl Fn(CR, CT) -> CF + Send + Sync + 'static,
    ) -> ClientResult<ResultOfProcessMessage>
    where
        CF: Future<Output = ()> + Send + Sync + 'static,
        CT: FromPrimitive,
        CR: DeserializeOwned,
    {
        let process = self.wrap_async_callback(
            crate::json_interface::processing::process_message,
            ProcessingModule::api(),
            crate::json_interface::processing::process_message_api(),
        );
        process.call_with_callback(params, callback).await
    }

    pub(crate) fn process_message_sync(
        &self,
        params: ParamsOfProcessMessage,
    ) -> ClientResult<ResultOfProcessMessage> {
        self.request("processing.process_message", params)
    }

    pub(crate) async fn fetch_account(&self, address: &str) -> Value {
        let mut result: ResultOfQuery = self.request_async(
            "net.query",
            ParamsOfQuery {
                query: "query account($address:String!){blockchain{account(address:$address){info{boc}}}}".to_owned(),
                variables: Some(json!({
                    "address": address.to_string(),
                })),
            })
            .await
            .unwrap();
        result
            .result
            .pointer_mut("/data/blockchain/account/info")
            .unwrap()
            .take()
    }

    pub(crate) async fn net_process_function(
        &self,
        address: String,
        abi: Abi,
        function_name: &str,
        input: Value,
        signer: Signer,
    ) -> ClientResult<ResultOfProcessMessage> {
        self.net_process_message(
            ParamsOfProcessMessage {
                message_encode_params: ParamsOfEncodeMessage {
                    address: Some(address),
                    abi,
                    call_set: Some(CallSet {
                        header: None,
                        function_name: function_name.into(),
                        input: Some(input),
                    }),
                    signer,
                    ..Default::default()
                },
                send_events: false,
            },
            Self::default_callback,
        )
        .await
    }

    pub(crate) async fn get_tokens_from_giver_async(
        &self,
        account: &str,
        value: Option<u64>,
    ) -> ResultOfProcessMessage {
        let giver_exists: ResultOfQuery = self
            .request_async(
                "net.query",
                ParamsOfQuery {
                    query: r#"query($addr: String!) {
                        blockchain {
                            account(address: $addr) {
                                info {
                                    acc_type
                                }
                            }
                        }
                    }"#
                    .to_string(),
                    variables: Some(json!({"addr": self.giver_address().await})),
                },
            )
            .await
            .unwrap_or_default();

        if giver_exists.result["data"]["blockchain"]["account"]["info"]["acc_type"]
            .as_i64()
            .unwrap_or_default()
            != 1
        {
            panic!("The giver contract should be deployed and active");
        }

        let (function, input) = match env::giver_type().as_str() {
            "v1" => (
                "sendGrams",
                json!({
                    "dest": account.to_string(),
                    "amount": value.unwrap_or(500_000_000u64),
                }),
            ),
            "v2" | "v3" => (
                "sendTransaction",
                json!({
                    "dest": account.to_string(),
                    "value": value.unwrap_or(500_000_000u64),
                    "bounce": false,
                }),
            ),
            _ => panic!("Unknown giver version"),
        };
        let run_result = self
            .net_process_function(
                self.giver_address().await,
                Self::giver_abi(),
                function,
                input,
                Signer::Keys {
                    keys: Self::giver_keys(),
                },
            )
            .await
            .unwrap();

        if run_result.transaction["out_msgs"][0].is_null() {
            panic!("The giver's topup call should result in at least 1 internal outbound message");
        }

        // wait for tokens reception
        let _: ResultOfQueryTransactionTree = self
            .request_async(
                "net.query_transaction_tree",
                ParamsOfQueryTransactionTree {
                    in_msg: run_result.transaction["in_msg"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        run_result
    }

    pub(crate) fn get_tokens_from_giver_sync(
        &self,
        account: &str,
        value: Option<u64>,
    ) -> ResultOfProcessMessage {
        self.context()
            .clone()
            .env
            .block_on(self.get_tokens_from_giver_async(account, value))
    }

    pub(crate) async fn deploy_with_giver_async(
        &self,
        params: ParamsOfEncodeMessage,
        value: Option<u64>,
    ) -> String {
        let msg = self.encode_message(params.clone()).await.unwrap();

        self.get_tokens_from_giver_async(&msg.address, value).await;

        let _ = self
            .net_process_message(
                ParamsOfProcessMessage {
                    message_encode_params: params,
                    send_events: false,
                },
                Self::default_callback,
            )
            .await
            .unwrap();

        msg.address
    }

    pub(crate) fn generate_sign_keys(&self) -> KeyPair {
        self.request("crypto.generate_random_sign_keys", ())
            .unwrap()
    }

    pub fn sign_detached(&self, data: &str, keys: &KeyPair) -> String {
        let sign_keys: KeyPair = self
            .request(
                "crypto.nacl_sign_keypair_from_secret_key",
                ParamsOfNaclSignKeyPairFromSecret {
                    secret: keys.secret.clone(),
                },
            )
            .unwrap();
        let result: ResultOfNaclSignDetached = self
            .request(
                "crypto.nacl_sign_detached",
                ParamsOfNaclSignDetached {
                    unsigned: data.into(),
                    secret: sign_keys.secret.clone(),
                },
            )
            .unwrap();
        result.signature
    }

    pub async fn resolve_app_request(&self, app_request_id: u32, result: impl Serialize) {
        self.request_async::<_, ()>(
            "client.resolve_app_request",
            ParamsOfResolveAppRequest {
                app_request_id,
                result: AppRequestResult::Ok {
                    result: json!(result),
                },
            },
        )
        .await
        .unwrap();
    }

    pub fn context(&self) -> Arc<ClientContext> {
        Runtime::required_context(self.context)
            .expect("Unable to get current ClientContext by handle")
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
