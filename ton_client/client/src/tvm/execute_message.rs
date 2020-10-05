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
 *
 */

use crate::boc::internal::{deserialize_cell_from_base64, deserialize_object_from_base64};
use crate::client::ClientContext;
use crate::error::{ApiError, ApiResult};
use crate::tvm::Error;
use ton_executor::BlockchainConfig;

use crate::abi::encode_message;
use crate::processing::{DecodedOutput, MessageSource};
use serde_json::Value;
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, ApiType, Clone, Default)]
pub struct ExecutionOptions {
    /// boc with config
    pub blockchain_config: Option<String>,
    /// time that is used as transaction time
    pub block_time: Option<u32>,
    /// block logical time
    pub block_lt: Option<u64>,
    /// transaction logical time
    pub transaction_lt: Option<u64>,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub enum ExecutionMode {
    /// Executes all phases and performs all checks
    Full,
    /// Executes contract only on TVM (part of compute phase)
    TvmOnly,
}

#[derive(Serialize, Deserialize, ApiType, Clone)]
pub struct ParamsOfExecuteMessage {
    /// Input message.
    pub message: MessageSource,
    /// Account BOC. Must be encoded as base64.
    pub account: String,
    /// Execution mode.
    pub mode: ExecutionMode,
    /// Execution options.
    pub execution_options: Option<ExecutionOptions>,
}

#[derive(Serialize, Deserialize, ApiType, Debug, PartialEq, Clone)]
pub struct ResultOfExecuteMessage {
    /// Parsed transaction.
    ///
    /// In addition to the regular transaction fields there is a
    /// `boc` field encoded with `base64` which contains source
    /// transaction BOC.
    pub transaction: Option<Value>,

    /// List of parsed output messages.
    ///
    /// Similar to the `transaction` each message contains the `boc`
    /// field.
    pub out_messages: Vec<Value>,

    /// Optional decoded message bodies according to the optional
    /// `abi` parameter.
    pub decoded: Option<DecodedOutput>,

    /// JSON with parsed updated account state. Attention! When used in
    /// `TvmOnly` mode only data in account state is updated.
    pub account: Option<Value>,
}

use ton_block;
use ton_block::MsgAddressInt;

#[api_function]
pub async fn execute_message(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfExecuteMessage,
) -> ApiResult<ResultOfExecuteMessage> {
    let (_, account) = deserialize_cell_from_base64(&params.account, "account")?;
    let (message, abi) = params.message.encode(&context)?;

    let message = deserialize_object_from_base64::<ton_block::Message>(&message, "message")?.object;
    let result = match params.mode {
        ExecutionMode::Full => crate::tvm::execute_message_tvm_only::execute_message_tvm_only(
            &context, account, message, options,
        ),
        ExecutionMode::TvmOnly => {
            crate::tvm::execute_message_full::execute_message_full(
                &context, account, message, options,
            )
            .await
        }
    }?;

    Ok(ResultOfExecuteMessage {
        transaction: None,
        out_messages: vec![],
        account: None,
        decoded: None,
    })
}
